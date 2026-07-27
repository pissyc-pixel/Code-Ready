use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};

use crate::application::snapshot_store::{SnapshotMutationError, SnapshotStore};
use crate::domain::contracts::{
    DetectionEventEnvelope, DetectionRunErrorCode, DetectionRunStatus, ToolId,
};
use crate::domain::detection::{DetectionSelectionError, validate_detection_selection};
use crate::platform::PlatformAdapter;
use crate::platform::process::ProcessRunner;
use crate::tools::shared::{Clock, DetectionContext, ToolDetector};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct EventPublishError;

pub trait DetectionEventSink: Send + Sync {
    fn publish(&self, event: DetectionEventEnvelope) -> Result<(), EventPublishError>;
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum StartDetectionError {
    InvalidSelection(DetectionSelectionError),
    AlreadyRunning(String),
    Internal,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ThreadSpawnError;

pub trait ThreadSpawner: Send + Sync {
    fn spawn(
        &self,
        name: String,
        task: Box<dyn FnOnce() + Send + 'static>,
    ) -> Result<(), ThreadSpawnError>;
}

#[derive(Clone, Copy, Debug, Default)]
struct NativeThreadSpawner;

impl ThreadSpawner for NativeThreadSpawner {
    fn spawn(
        &self,
        name: String,
        task: Box<dyn FnOnce() + Send + 'static>,
    ) -> Result<(), ThreadSpawnError> {
        std::thread::Builder::new()
            .name(name)
            .spawn(task)
            .map(|_| ())
            .map_err(|_| ThreadSpawnError)
    }
}

pub struct DetectionService {
    store: Arc<SnapshotStore>,
    platform: Arc<dyn PlatformAdapter>,
    runner: Arc<dyn ProcessRunner>,
    clock: Arc<dyn Clock>,
    detectors: Arc<Vec<Arc<dyn ToolDetector>>>,
    sink: Arc<dyn DetectionEventSink>,
    spawner: Arc<dyn ThreadSpawner>,
    next_run_id: AtomicU64,
}

impl DetectionService {
    pub fn new(
        store: Arc<SnapshotStore>,
        platform: Arc<dyn PlatformAdapter>,
        runner: Arc<dyn ProcessRunner>,
        clock: Arc<dyn Clock>,
        detectors: Vec<Box<dyn ToolDetector>>,
        sink: Arc<dyn DetectionEventSink>,
    ) -> Self {
        Self::with_thread_spawner(
            store,
            platform,
            runner,
            clock,
            detectors,
            sink,
            Arc::new(NativeThreadSpawner),
        )
    }

    pub fn with_thread_spawner(
        store: Arc<SnapshotStore>,
        platform: Arc<dyn PlatformAdapter>,
        runner: Arc<dyn ProcessRunner>,
        clock: Arc<dyn Clock>,
        detectors: Vec<Box<dyn ToolDetector>>,
        sink: Arc<dyn DetectionEventSink>,
        spawner: Arc<dyn ThreadSpawner>,
    ) -> Self {
        let detectors = detectors
            .into_iter()
            .map(|detector| Arc::from(detector) as Arc<dyn ToolDetector>)
            .collect();
        Self {
            store,
            platform,
            runner,
            clock,
            detectors: Arc::new(detectors),
            sink,
            spawner,
            next_run_id: AtomicU64::new(1),
        }
    }

    pub fn start(&self, requested: Option<Vec<ToolId>>) -> Result<String, StartDetectionError> {
        let requested = validate_detection_selection(requested)
            .map_err(StartDetectionError::InvalidSelection)?;
        let run_number = self
            .next_run_id
            .fetch_update(Ordering::SeqCst, Ordering::SeqCst, |current| {
                current.checked_add(1)
            })
            .map_err(|_| StartDetectionError::Internal)?;
        let run_id = format!("run-{run_number}");

        let begin_event = match self.store.begin_run(&run_id, requested.clone()) {
            Ok(event) => event,
            Err(SnapshotMutationError::RunAlreadyActive) => {
                let active_run_id = self
                    .store
                    .snapshot()
                    .detection_run
                    .filter(|run| run.status == DetectionRunStatus::Running)
                    .map(|run| run.id)
                    .unwrap_or_else(|| "unknown".to_owned());
                return Err(StartDetectionError::AlreadyRunning(active_run_id));
            }
            Err(_) => return Err(StartDetectionError::Internal),
        };
        publish_best_effort(self.sink.as_ref(), begin_event);

        let worker_store = Arc::clone(&self.store);
        let worker_platform = Arc::clone(&self.platform);
        let worker_runner = Arc::clone(&self.runner);
        let worker_clock = Arc::clone(&self.clock);
        let worker_detectors = Arc::clone(&self.detectors);
        let worker_sink = Arc::clone(&self.sink);
        let worker_run_id = run_id.clone();
        let worker = Box::new(move || {
            run_detection(DetectionWorker {
                run_id: worker_run_id,
                requested,
                store: worker_store,
                platform: worker_platform,
                runner: worker_runner,
                clock: worker_clock,
                detectors: worker_detectors,
                sink: worker_sink,
            });
        });

        if self
            .spawner
            .spawn(format!("code-ready-detect-{run_id}"), worker)
            .is_err()
        {
            if let Ok(event) = self
                .store
                .fail_run(&run_id, DetectionRunErrorCode::Internal)
            {
                publish_best_effort(self.sink.as_ref(), event);
            }
            return Err(StartDetectionError::Internal);
        }

        Ok(run_id)
    }
}

struct DetectionWorker {
    run_id: String,
    requested: Vec<ToolId>,
    store: Arc<SnapshotStore>,
    platform: Arc<dyn PlatformAdapter>,
    runner: Arc<dyn ProcessRunner>,
    clock: Arc<dyn Clock>,
    detectors: Arc<Vec<Arc<dyn ToolDetector>>>,
    sink: Arc<dyn DetectionEventSink>,
}

fn run_detection(worker: DetectionWorker) {
    for tool_id in worker.requested {
        let Some(detector) = worker
            .detectors
            .iter()
            .find(|detector| detector.tool_id() == tool_id)
            .cloned()
        else {
            fail_detection_run(&worker.store, worker.sink.as_ref(), &worker.run_id);
            return;
        };

        let context = DetectionContext {
            platform: worker.platform.as_ref(),
            runner: worker.runner.as_ref(),
            clock: worker.clock.as_ref(),
        };
        let observation = detector.detect(&context);
        match worker.store.record_observation(&worker.run_id, observation) {
            Ok(event) => publish_best_effort(worker.sink.as_ref(), event),
            Err(_) => {
                fail_detection_run(&worker.store, worker.sink.as_ref(), &worker.run_id);
                return;
            }
        }
    }

    if let Ok(event) = worker.store.finish_run(&worker.run_id) {
        publish_best_effort(worker.sink.as_ref(), event);
    } else {
        fail_detection_run(&worker.store, worker.sink.as_ref(), &worker.run_id);
    }
}

fn fail_detection_run(store: &SnapshotStore, sink: &dyn DetectionEventSink, run_id: &str) {
    if let Ok(event) = store.fail_run(run_id, DetectionRunErrorCode::Internal) {
        publish_best_effort(sink, event);
    }
}

fn publish_best_effort(sink: &dyn DetectionEventSink, event: DetectionEventEnvelope) {
    let _ = sink.publish(event);
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::application::snapshot_store::SnapshotStore;
    use crate::domain::contracts::{
        DetectionEvidence, DetectionEvidenceCode, DetectionRunStatus, ObservedToolState,
        PlatformId, ProcessExitKind, ToolId, ToolObservation, VersionStatus,
    };
    use crate::domain::tool_registry::built_in_tool_registry;
    use crate::platform::fake::FakePlatformAdapter;
    use crate::platform::fake_process::FakeProcessRunner;
    use crate::platform::process::ProcessOutcome;
    use crate::platform::{CandidateOrigin, ExecutableCandidate, PlatformAdapter};
    use crate::tools::built_in_detector_registry;
    use std::collections::VecDeque;
    use std::sync::Mutex;
    use std::time::Duration;

    struct FixedClock;

    impl Clock for FixedClock {
        fn now_epoch_ms(&self) -> u64 {
            1_754_000_000_000
        }
    }

    #[derive(Default)]
    struct RecordingEventSink {
        events: Mutex<Vec<DetectionEventEnvelope>>,
    }

    impl RecordingEventSink {
        fn sequences(&self) -> Vec<u64> {
            self.events
                .lock()
                .expect("event mutex is not poisoned")
                .iter()
                .map(|event| event.sequence)
                .collect()
        }
    }

    impl DetectionEventSink for RecordingEventSink {
        fn publish(&self, event: DetectionEventEnvelope) -> Result<(), EventPublishError> {
            self.events
                .lock()
                .expect("event mutex is not poisoned")
                .push(event);
            Ok(())
        }
    }

    struct AlwaysFailingEventSink;

    impl DetectionEventSink for AlwaysFailingEventSink {
        fn publish(&self, _event: DetectionEventEnvelope) -> Result<(), EventPublishError> {
            Err(EventPublishError)
        }
    }

    struct NoopThreadSpawner;

    impl ThreadSpawner for NoopThreadSpawner {
        fn spawn(
            &self,
            _name: String,
            _task: Box<dyn FnOnce() + Send + 'static>,
        ) -> Result<(), ThreadSpawnError> {
            Ok(())
        }
    }

    struct FailingThreadSpawner;

    impl ThreadSpawner for FailingThreadSpawner {
        fn spawn(
            &self,
            _name: String,
            _task: Box<dyn FnOnce() + Send + 'static>,
        ) -> Result<(), ThreadSpawnError> {
            Err(ThreadSpawnError)
        }
    }

    struct DetectionHarness {
        service: DetectionService,
        store: Arc<SnapshotStore>,
        events: Arc<RecordingEventSink>,
    }

    struct DetectionHarnessBuilder {
        platform: PlatformId,
        sink: Arc<dyn DetectionEventSink>,
        events: Arc<RecordingEventSink>,
        spawner: Arc<dyn ThreadSpawner>,
    }

    impl DetectionHarness {
        fn builder(platform: PlatformId) -> DetectionHarnessBuilder {
            let events = Arc::new(RecordingEventSink::default());
            DetectionHarnessBuilder {
                platform,
                sink: events.clone(),
                events,
                spawner: Arc::new(NativeThreadSpawner),
            }
        }

        fn with_sink<S>(sink: S) -> DetectionHarnessBuilder
        where
            S: DetectionEventSink + 'static,
        {
            let events = Arc::new(RecordingEventSink::default());
            DetectionHarnessBuilder {
                platform: PlatformId::MacosArm64,
                sink: Arc::new(sink),
                events,
                spawner: Arc::new(NativeThreadSpawner),
            }
        }

        fn blocked_runner() -> DetectionHarness {
            Self::builder(PlatformId::MacosArm64)
                .with_thread_spawner(Arc::new(NoopThreadSpawner))
                .with_healthy_versions([
                    (ToolId::Git, "git version 2.47.1"),
                    (ToolId::ClaudeCode, "2.1.89 (Claude Code)"),
                    (ToolId::CodexCli, "codex-cli 0.138.0"),
                ])
        }

        fn wait_until_completed(&self, run_id: &str) {
            for _ in 0..1_000 {
                let status = self
                    .store
                    .snapshot()
                    .detection_run
                    .as_ref()
                    .map(|run| run.status.clone());
                if status == Some(DetectionRunStatus::Completed)
                    || status == Some(DetectionRunStatus::Failed)
                {
                    return;
                }
                std::thread::sleep(Duration::from_millis(1));
            }
            panic!("detection run {run_id} did not finish");
        }
    }

    impl DetectionHarnessBuilder {
        fn with_thread_spawner(mut self, spawner: Arc<dyn ThreadSpawner>) -> Self {
            self.spawner = spawner;
            self
        }

        fn with_healthy_versions(self, versions: [(ToolId, &'static str); 3]) -> DetectionHarness {
            let mut adapter_builder = FakePlatformAdapter::builder(self.platform.clone());
            let mut outcomes = VecDeque::new();
            for (tool_id, version) in versions {
                let executable = format!("/fake/{}", tool_id_name(&tool_id));
                adapter_builder = adapter_builder.with_candidates(
                    tool_id.clone(),
                    vec![ExecutableCandidate::native(
                        executable.clone(),
                        executable,
                        CandidateOrigin::Path,
                    )],
                );
                outcomes.push_back(success(version));
            }
            self.build(adapter_builder.build(), outcomes)
        }

        fn with_codex_broken_and_existing_git(self) -> DetectionHarness {
            let adapter = FakePlatformAdapter::builder(self.platform.clone())
                .with_candidates(
                    ToolId::CodexCli,
                    vec![ExecutableCandidate::native(
                        "/fake/codex",
                        "/fake/codex",
                        CandidateOrigin::Path,
                    )],
                )
                .build();
            let clock: Arc<dyn Clock> = Arc::new(FixedClock);
            let store = Arc::new(SnapshotStore::new(
                self.platform.clone(),
                built_in_tool_registry(),
                clock.clone(),
            ));
            store.begin_run("seed", vec![ToolId::Git]).unwrap();
            store
                .record_observation("seed", healthy_git("2.47.1"))
                .unwrap();
            store.finish_run("seed").unwrap();
            let runner: Arc<dyn ProcessRunner> =
                Arc::new(FakeProcessRunner::new([ProcessOutcome::Exited {
                    code: Some(23),
                    stdout: vec![],
                    stderr: vec![],
                    stdout_truncated: false,
                    stderr_truncated: false,
                }]));
            self.build_with_parts(store, Arc::new(adapter), runner, clock)
        }

        fn with_spawn_failure(self) -> DetectionHarness {
            self.with_thread_spawner(Arc::new(FailingThreadSpawner))
                .with_healthy_versions([
                    (ToolId::Git, "git version 2.47.1"),
                    (ToolId::ClaudeCode, "2.1.89 (Claude Code)"),
                    (ToolId::CodexCli, "codex-cli 0.138.0"),
                ])
        }

        fn build(
            self,
            adapter: FakePlatformAdapter,
            outcomes: VecDeque<ProcessOutcome>,
        ) -> DetectionHarness {
            let clock: Arc<dyn Clock> = Arc::new(FixedClock);
            let store = Arc::new(SnapshotStore::new(
                self.platform.clone(),
                built_in_tool_registry(),
                clock.clone(),
            ));
            let runner: Arc<dyn ProcessRunner> = Arc::new(FakeProcessRunner::new(outcomes));
            self.build_with_parts(store, Arc::new(adapter), runner, clock)
        }

        fn build_with_parts(
            self,
            store: Arc<SnapshotStore>,
            platform: Arc<dyn PlatformAdapter>,
            runner: Arc<dyn ProcessRunner>,
            clock: Arc<dyn Clock>,
        ) -> DetectionHarness {
            let service = DetectionService::with_thread_spawner(
                store.clone(),
                platform,
                runner,
                clock,
                built_in_detector_registry(),
                self.sink,
                self.spawner,
            );
            DetectionHarness {
                service,
                store,
                events: self.events,
            }
        }
    }

    fn tool_id_name(tool_id: &ToolId) -> &'static str {
        match tool_id {
            ToolId::Git => "git",
            ToolId::ClaudeCode => "claude",
            ToolId::CodexCli => "codex",
            ToolId::Winget | ToolId::Nodejs => "unsupported",
        }
    }

    fn success(version: &str) -> ProcessOutcome {
        ProcessOutcome::Exited {
            code: Some(0),
            stdout: version.as_bytes().to_vec(),
            stderr: vec![],
            stdout_truncated: false,
            stderr_truncated: false,
        }
    }

    fn healthy_git(version: &str) -> ToolObservation {
        ToolObservation {
            tool_id: ToolId::Git,
            state: ObservedToolState::PresentHealthy,
            version: Some(version.to_owned()),
            version_status: VersionStatus::NotComparable,
            evidence: DetectionEvidence {
                code: DetectionEvidenceCode::PathCommandHealthy,
                display_path: Some("/usr/bin/git".to_owned()),
                exit: Some(ProcessExitKind::Success),
            },
            checked_at_epoch_ms: 1_754_000_000_000,
        }
    }

    fn observation(
        snapshot: &crate::domain::contracts::AppSnapshot,
        tool_id: ToolId,
    ) -> &ToolObservation {
        snapshot
            .observations
            .iter()
            .find(|observation| observation.tool_id == tool_id)
            .expect("observation exists")
    }

    #[test]
    fn full_detection_returns_immediately_then_publishes_snapshot_events_in_order() {
        let harness = DetectionHarness::builder(PlatformId::MacosArm64).with_healthy_versions([
            (ToolId::Git, "git version 2.47.1"),
            (ToolId::ClaudeCode, "2.1.89 (Claude Code)"),
            (ToolId::CodexCli, "codex-cli 0.138.0"),
        ]);

        let run_id = harness.service.start(None).unwrap();
        harness.wait_until_completed(&run_id);

        let snapshot = harness.store.snapshot();
        assert_eq!(
            snapshot
                .observations
                .iter()
                .map(|item| item.tool_id.clone())
                .collect::<Vec<_>>(),
            vec![ToolId::Git, ToolId::ClaudeCode, ToolId::CodexCli]
        );
        assert_eq!(
            snapshot.detection_run.unwrap().status,
            DetectionRunStatus::Completed
        );
        assert_eq!(harness.events.sequences(), vec![1, 2, 3, 4, 5]);
    }

    #[test]
    fn dropped_or_failed_event_delivery_never_rolls_back_snapshot_truth() {
        let harness = DetectionHarness::with_sink(AlwaysFailingEventSink).with_healthy_versions([
            (ToolId::Git, "git version 2.47.1"),
            (ToolId::ClaudeCode, "2.1.89 (Claude Code)"),
            (ToolId::CodexCli, "codex-cli 0.138.0"),
        ]);
        let run_id = harness.service.start(Some(vec![ToolId::Git])).unwrap();
        harness.wait_until_completed(&run_id);

        let snapshot = harness.store.snapshot();
        assert_eq!(snapshot.observations.len(), 1);
        assert_eq!(snapshot.last_event_sequence, 3);
        assert_eq!(snapshot.snapshot_version, 4);
    }

    #[test]
    fn a_second_run_is_rejected_without_changing_tool_facts() {
        let harness = DetectionHarness::blocked_runner();
        let first = harness.service.start(Some(vec![ToolId::Git])).unwrap();
        assert_eq!(
            harness.service.start(Some(vec![ToolId::CodexCli])),
            Err(StartDetectionError::AlreadyRunning(first))
        );
        assert!(harness.store.snapshot().observations.is_empty());
    }

    #[test]
    fn on_demand_detection_touches_only_requested_tools() {
        let harness =
            DetectionHarness::builder(PlatformId::WindowsX64).with_codex_broken_and_existing_git();
        let run_id = harness.service.start(Some(vec![ToolId::CodexCli])).unwrap();
        harness.wait_until_completed(&run_id);

        let snapshot = harness.store.snapshot();
        assert_eq!(
            observation(&snapshot, ToolId::Git).version.as_deref(),
            Some("2.47.1")
        );
        assert_eq!(
            observation(&snapshot, ToolId::CodexCli).state,
            ObservedToolState::PresentBroken
        );
        assert!(
            !snapshot
                .observations
                .iter()
                .any(|item| item.tool_id == ToolId::ClaudeCode)
        );
    }

    #[test]
    fn thread_spawn_failure_fails_the_run_without_tool_observations() {
        let harness = DetectionHarness::builder(PlatformId::MacosArm64).with_spawn_failure();

        assert_eq!(
            harness.service.start(Some(vec![ToolId::Git])),
            Err(StartDetectionError::Internal)
        );
        let snapshot = harness.store.snapshot();
        let run = snapshot.detection_run.expect("failed run is retained");
        assert_eq!(run.status, DetectionRunStatus::Failed);
        assert_eq!(run.error_code, Some(DetectionRunErrorCode::Internal));
        assert!(snapshot.observations.is_empty());
    }
}
