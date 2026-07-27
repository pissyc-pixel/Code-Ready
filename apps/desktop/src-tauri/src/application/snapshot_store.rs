use std::sync::{Arc, Mutex};

use crate::domain::contracts::{
    AppSnapshot, DetectionEventEnvelope, DetectionEventType, DetectionRun, DetectionRunErrorCode,
    DetectionRunStatus, PlatformId, ToolDefinition, ToolId, ToolObservation,
};
use crate::domain::detection::DETECTABLE_TOOL_IDS;
use crate::tools::shared::Clock;

pub struct SnapshotStore {
    inner: Mutex<StoreState>,
    clock: Arc<dyn Clock>,
}

struct StoreState {
    snapshot: AppSnapshot,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum SnapshotMutationError {
    RunAlreadyActive,
    RunNotActive,
    RunIdMismatch,
    ToolNotRequested,
    CounterOverflow,
}

impl SnapshotStore {
    pub fn new(platform: PlatformId, tools: Vec<ToolDefinition>, clock: Arc<dyn Clock>) -> Self {
        Self {
            inner: Mutex::new(StoreState {
                snapshot: AppSnapshot {
                    schema_version: 2,
                    snapshot_version: 1,
                    last_event_sequence: 0,
                    platform,
                    tools,
                    observations: vec![],
                    detection_run: None,
                },
            }),
            clock,
        }
    }

    pub fn snapshot(&self) -> AppSnapshot {
        self.inner
            .lock()
            .expect("snapshot store mutex is not poisoned")
            .snapshot
            .clone()
    }

    pub fn begin_run(
        &self,
        run_id: &str,
        tool_ids: Vec<ToolId>,
    ) -> Result<DetectionEventEnvelope, SnapshotMutationError> {
        let mut state = self
            .inner
            .lock()
            .expect("snapshot store mutex is not poisoned");
        if state
            .snapshot
            .detection_run
            .as_ref()
            .is_some_and(|run| run.status == DetectionRunStatus::Running)
        {
            return Err(SnapshotMutationError::RunAlreadyActive);
        }

        let next_versions = next_versions(&state.snapshot)?;
        let started_at_epoch_ms = self.clock.now_epoch_ms();
        state.snapshot.detection_run = Some(DetectionRun {
            id: run_id.to_owned(),
            requested_tool_ids: tool_ids,
            status: DetectionRunStatus::Running,
            started_at_epoch_ms,
            finished_at_epoch_ms: None,
            error_code: None,
        });
        self.commit_mutation(&mut state, run_id, next_versions)
    }

    pub fn record_observation(
        &self,
        run_id: &str,
        observation: ToolObservation,
    ) -> Result<DetectionEventEnvelope, SnapshotMutationError> {
        let mut state = self
            .inner
            .lock()
            .expect("snapshot store mutex is not poisoned");
        let run = active_run(&state.snapshot, run_id)?;
        if !run.requested_tool_ids.contains(&observation.tool_id) {
            return Err(SnapshotMutationError::ToolNotRequested);
        }

        let next_versions = next_versions(&state.snapshot)?;
        if let Some(existing) = state
            .snapshot
            .observations
            .iter_mut()
            .find(|existing| existing.tool_id == observation.tool_id)
        {
            *existing = observation;
        } else {
            state.snapshot.observations.push(observation);
        }
        state.snapshot.observations.sort_by_key(|item| {
            DETECTABLE_TOOL_IDS
                .iter()
                .position(|tool_id| tool_id == &item.tool_id)
                .unwrap_or(usize::MAX)
        });
        self.commit_mutation(&mut state, run_id, next_versions)
    }

    pub fn finish_run(
        &self,
        run_id: &str,
    ) -> Result<DetectionEventEnvelope, SnapshotMutationError> {
        let mut state = self
            .inner
            .lock()
            .expect("snapshot store mutex is not poisoned");
        active_run(&state.snapshot, run_id)?;
        let next_versions = next_versions(&state.snapshot)?;
        let finished_at_epoch_ms = self.clock.now_epoch_ms();
        let run = state
            .snapshot
            .detection_run
            .as_mut()
            .expect("active run was checked above");
        run.status = DetectionRunStatus::Completed;
        run.finished_at_epoch_ms = Some(finished_at_epoch_ms);
        run.error_code = None;
        self.commit_mutation(&mut state, run_id, next_versions)
    }

    pub fn fail_run(
        &self,
        run_id: &str,
        error_code: DetectionRunErrorCode,
    ) -> Result<DetectionEventEnvelope, SnapshotMutationError> {
        let mut state = self
            .inner
            .lock()
            .expect("snapshot store mutex is not poisoned");
        active_run(&state.snapshot, run_id)?;
        let next_versions = next_versions(&state.snapshot)?;
        let finished_at_epoch_ms = self.clock.now_epoch_ms();
        let run = state
            .snapshot
            .detection_run
            .as_mut()
            .expect("active run was checked above");
        run.status = DetectionRunStatus::Failed;
        run.finished_at_epoch_ms = Some(finished_at_epoch_ms);
        run.error_code = Some(error_code);
        self.commit_mutation(&mut state, run_id, next_versions)
    }

    fn commit_mutation(
        &self,
        state: &mut StoreState,
        run_id: &str,
        (snapshot_version, sequence): (u64, u64),
    ) -> Result<DetectionEventEnvelope, SnapshotMutationError> {
        let emitted_at_epoch_ms = self.clock.now_epoch_ms();

        state.snapshot.snapshot_version = snapshot_version;
        state.snapshot.last_event_sequence = sequence;
        Ok(DetectionEventEnvelope {
            schema_version: 1,
            sequence,
            snapshot_version,
            emitted_at_epoch_ms,
            event_type: DetectionEventType::DetectionChanged,
            run_id: run_id.to_owned(),
        })
    }
}

fn next_versions(snapshot: &AppSnapshot) -> Result<(u64, u64), SnapshotMutationError> {
    let snapshot_version = snapshot
        .snapshot_version
        .checked_add(1)
        .ok_or(SnapshotMutationError::CounterOverflow)?;
    let sequence = snapshot
        .last_event_sequence
        .checked_add(1)
        .ok_or(SnapshotMutationError::CounterOverflow)?;
    Ok((snapshot_version, sequence))
}

fn active_run<'a>(
    snapshot: &'a AppSnapshot,
    run_id: &str,
) -> Result<&'a DetectionRun, SnapshotMutationError> {
    let Some(run) = snapshot.detection_run.as_ref() else {
        return Err(SnapshotMutationError::RunNotActive);
    };
    if run.status != DetectionRunStatus::Running {
        return Err(SnapshotMutationError::RunNotActive);
    }
    if run.id != run_id {
        return Err(SnapshotMutationError::RunIdMismatch);
    }
    Ok(run)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::contracts::{
        DetectionEvidence, DetectionEvidenceCode, DetectionRunStatus, ObservedToolState,
        ProcessExitKind, ToolId, ToolObservation, VersionStatus,
    };
    use crate::domain::tool_registry::built_in_tool_registry;
    use crate::platform::process::SystemClock;
    use crate::tools::shared::Clock;
    use std::collections::VecDeque;
    use std::sync::{Arc, Mutex};

    struct FakeClock {
        values: Mutex<VecDeque<u64>>,
    }

    impl FakeClock {
        fn new(values: impl IntoIterator<Item = u64>) -> Self {
            Self {
                values: Mutex::new(values.into_iter().collect()),
            }
        }
    }

    impl Clock for FakeClock {
        fn now_epoch_ms(&self) -> u64 {
            self.values
                .lock()
                .expect("fake clock mutex is not poisoned")
                .pop_front()
                .unwrap_or(999)
        }
    }

    fn store_with_running_run(run_id: &str, tool_ids: [ToolId; 2]) -> SnapshotStore {
        let clock = Arc::new(FakeClock::new([100, 110, 120, 130]));
        let store = SnapshotStore::new(
            crate::domain::contracts::PlatformId::WindowsX64,
            built_in_tool_registry(),
            clock,
        );
        store
            .begin_run(run_id, tool_ids.into_iter().collect())
            .unwrap();
        store
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
            checked_at_epoch_ms: 110,
        }
    }

    fn broken_codex() -> ToolObservation {
        ToolObservation {
            tool_id: ToolId::CodexCli,
            state: ObservedToolState::PresentBroken,
            version: None,
            version_status: VersionStatus::Unknown,
            evidence: DetectionEvidence {
                code: DetectionEvidenceCode::CommandTimedOut,
                display_path: Some("~/.local/bin/codex".to_owned()),
                exit: Some(ProcessExitKind::TimedOut),
            },
            checked_at_epoch_ms: 120,
        }
    }

    #[test]
    fn snapshot_store_versions_each_mutation_and_never_changes_on_read() {
        let clock = Arc::new(FakeClock::new([100, 110, 120]));
        let store = SnapshotStore::new(
            crate::domain::contracts::PlatformId::WindowsX64,
            built_in_tool_registry(),
            clock,
        );

        let initial = store.snapshot();
        assert_eq!(initial.schema_version, 2);
        assert_eq!(initial.snapshot_version, 1);
        assert_eq!(initial.last_event_sequence, 0);
        assert!(initial.observations.is_empty());
        assert!(initial.detection_run.is_none());
        assert_eq!(store.snapshot(), initial);

        let started = store.begin_run("run-1", vec![ToolId::Git]).unwrap();
        assert_eq!(started.sequence, 1);
        assert_eq!(started.snapshot_version, 2);
        assert_eq!(store.snapshot().last_event_sequence, 1);
    }

    #[test]
    fn recording_an_observation_replaces_only_the_same_tool_fact() {
        let store = store_with_running_run("run-1", [ToolId::Git, ToolId::CodexCli]);
        store
            .record_observation("run-1", healthy_git("2.47.1"))
            .unwrap();
        store.record_observation("run-1", broken_codex()).unwrap();

        let snapshot = store.snapshot();
        assert_eq!(snapshot.observations.len(), 2);
        assert_eq!(snapshot.observations[0].tool_id, ToolId::Git);
        assert_eq!(snapshot.observations[1].tool_id, ToolId::CodexCli);
        assert_eq!(
            snapshot.detection_run.unwrap().status,
            DetectionRunStatus::Running
        );
    }

    #[test]
    fn counter_overflow_rejects_the_mutation_without_changing_the_snapshot() {
        let store = SnapshotStore::new(
            crate::domain::contracts::PlatformId::WindowsX64,
            built_in_tool_registry(),
            Arc::new(FakeClock::new([100, 110])),
        );
        {
            let mut state = store
                .inner
                .lock()
                .expect("snapshot store mutex is not poisoned");
            state.snapshot.snapshot_version = u64::MAX;
        }
        let before = store.snapshot();

        assert_eq!(
            store.begin_run("run-overflow", vec![ToolId::Git]),
            Err(SnapshotMutationError::CounterOverflow)
        );
        assert_eq!(store.snapshot(), before);
    }

    #[test]
    fn concurrent_reads_observe_complete_versioned_snapshots() {
        use std::sync::{Arc, Barrier};
        use std::thread;

        let store = Arc::new(SnapshotStore::new(
            crate::domain::contracts::PlatformId::WindowsX64,
            built_in_tool_registry(),
            Arc::new(FakeClock::new(std::iter::repeat_n(100, 300))),
        ));
        store
            .begin_run("run-concurrent", vec![ToolId::Git, ToolId::CodexCli])
            .unwrap();

        let start = Arc::new(Barrier::new(9));
        let mut readers = Vec::new();
        for _ in 0..8 {
            let store = Arc::clone(&store);
            let start = Arc::clone(&start);
            readers.push(thread::spawn(move || {
                start.wait();
                for _ in 0..250 {
                    let snapshot = store.snapshot();
                    assert_eq!(
                        snapshot.snapshot_version,
                        snapshot
                            .last_event_sequence
                            .checked_add(1)
                            .expect("snapshot version invariant does not overflow")
                    );
                    let mut tool_ids = Vec::new();
                    for observation in &snapshot.observations {
                        assert!(!tool_ids.contains(&observation.tool_id));
                        tool_ids.push(observation.tool_id.clone());
                        match observation.state {
                            ObservedToolState::PresentHealthy
                            | ObservedToolState::PresentPathIssue => {
                                assert!(observation.version.is_some());
                                assert_eq!(
                                    observation.version_status,
                                    VersionStatus::NotComparable
                                );
                                assert_eq!(
                                    observation.evidence.exit,
                                    Some(ProcessExitKind::Success)
                                );
                            }
                            ObservedToolState::PresentBroken => {
                                assert!(observation.version.is_none());
                                assert_eq!(observation.version_status, VersionStatus::Unknown);
                            }
                            ObservedToolState::Absent | ObservedToolState::Unknown => {}
                        }
                    }
                }
            }));
        }

        start.wait();
        for index in 0..100 {
            let observation = if index % 2 == 0 {
                healthy_git(&format!("2.47.{index}"))
            } else {
                broken_codex()
            };
            let run_id = "run-concurrent";
            let _ = store.record_observation(run_id, observation).unwrap();
        }
        store.finish_run("run-concurrent").unwrap();

        for reader in readers {
            reader.join().expect("reader thread did not panic");
        }
    }

    #[allow(dead_code)]
    fn _system_clock_is_available_for_production_graph() {
        let _ = SystemClock;
    }
}
