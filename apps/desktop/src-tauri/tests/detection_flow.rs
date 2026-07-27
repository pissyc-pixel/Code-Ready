use std::sync::Arc;
use std::thread;
use std::time::Duration;

use code_ready_desktop_lib::application::detection::{
    DetectionEventSink, DetectionService, EventPublishError,
};
use code_ready_desktop_lib::application::snapshot_store::SnapshotStore;
use code_ready_desktop_lib::domain::contracts::{
    AppSnapshot, DetectionRunStatus, ObservedToolState, PlatformId, ToolId, VersionStatus,
};
use code_ready_desktop_lib::domain::tool_registry::built_in_tool_registry;
use code_ready_desktop_lib::platform::fake::FakePlatformAdapter;
use code_ready_desktop_lib::platform::fake_process::FakeProcessRunner;
use code_ready_desktop_lib::platform::process::{ProcessOutcome, ProcessRunner};
use code_ready_desktop_lib::platform::{CandidateOrigin, ExecutableCandidate, PlatformAdapter};
use code_ready_desktop_lib::tools::built_in_detector_registry;
use code_ready_desktop_lib::tools::shared::Clock;

struct FixedClock;

impl Clock for FixedClock {
    fn now_epoch_ms(&self) -> u64 {
        1_754_000_000_000
    }
}

struct NoopEventSink;

impl DetectionEventSink for NoopEventSink {
    fn publish(
        &self,
        _event: code_ready_desktop_lib::domain::contracts::DetectionEventEnvelope,
    ) -> Result<(), EventPublishError> {
        Ok(())
    }
}

struct EndToEndHarnessBuilder {
    platform: PlatformId,
    adapter: code_ready_desktop_lib::platform::fake::FakePlatformAdapterBuilder,
    outcomes: Vec<ProcessOutcome>,
}

struct EndToEndHarness {
    store: Arc<SnapshotStore>,
    service: DetectionService,
    runner: Arc<FakeProcessRunner>,
}

impl EndToEndHarnessBuilder {
    fn for_platform(platform: PlatformId) -> Self {
        EndToEndHarnessBuilder {
            platform: platform.clone(),
            adapter: FakePlatformAdapter::builder(platform),
            outcomes: vec![],
        }
    }
}

impl EndToEndHarness {
    fn bootstrap(&self) -> AppSnapshot {
        self.store.snapshot()
    }

    fn detect_all(
        &self,
    ) -> Result<String, code_ready_desktop_lib::application::detection::StartDetectionError> {
        self.service.start(None)
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
            thread::sleep(Duration::from_millis(1));
        }
        panic!("detection run {run_id} did not finish");
    }

    fn process_requests(&self) -> Vec<code_ready_desktop_lib::platform::process::ProcessRequest> {
        self.runner.requests()
    }

    fn privilege_requests(&self) -> usize {
        0
    }

    fn network_requests(&self) -> usize {
        0
    }
}

impl EndToEndHarnessBuilder {
    fn with_path_native(mut self, tool_id: ToolId, output: &str) -> Self {
        let path = format!("/fake/{}", tool_name(&tool_id));
        self.adapter = self.adapter.with_candidates(
            tool_id,
            vec![ExecutableCandidate::native(
                path.clone(),
                path,
                CandidateOrigin::Path,
            )],
        );
        self.outcomes.push(success(output));
        self
    }

    fn with_known_native(mut self, tool_id: ToolId, output: &str) -> Self {
        let name = tool_name(&tool_id);
        let path = format!("/fake/{name}");
        self.adapter = self.adapter.with_candidates(
            tool_id,
            vec![ExecutableCandidate::native(
                path,
                format!("~/.local/bin/{name}"),
                CandidateOrigin::KnownLocation,
            )],
        );
        self.outcomes.push(success(output));
        self
    }

    fn without_candidate(self, _tool_id: ToolId) -> EndToEndHarness {
        self.build()
    }

    fn with_all_absent(self) -> EndToEndHarness {
        self.build()
    }

    fn build(self) -> EndToEndHarness {
        let adapter: Arc<dyn PlatformAdapter> = Arc::new(self.adapter.build());
        let runner = Arc::new(FakeProcessRunner::new(self.outcomes));
        let runner_for_service: Arc<dyn ProcessRunner> = runner.clone();
        let clock: Arc<dyn Clock> = Arc::new(FixedClock);
        let store = Arc::new(SnapshotStore::new(
            self.platform,
            built_in_tool_registry(),
            clock.clone(),
        ));
        let service = DetectionService::new(
            store.clone(),
            adapter,
            runner_for_service,
            clock,
            built_in_detector_registry(),
            Arc::new(NoopEventSink),
        );
        EndToEndHarness {
            store,
            service,
            runner,
        }
    }
}

fn tool_name(tool_id: &ToolId) -> &'static str {
    match tool_id {
        ToolId::Git => "git",
        ToolId::ClaudeCode => "claude",
        ToolId::CodexCli => "codex",
        ToolId::Winget | ToolId::Nodejs => "unsupported",
    }
}

fn success(output: &str) -> ProcessOutcome {
    ProcessOutcome::Exited {
        code: Some(0),
        stdout: output.as_bytes().to_vec(),
        stderr: vec![],
        stdout_truncated: false,
        stderr_truncated: false,
    }
}

fn facts(snapshot: &AppSnapshot) -> Vec<(ToolId, ObservedToolState, VersionStatus)> {
    snapshot
        .observations
        .iter()
        .map(|observation| {
            (
                observation.tool_id.clone(),
                observation.state.clone(),
                observation.version_status.clone(),
            )
        })
        .collect()
}

#[test]
fn windows_and_macos_fake_flows_produce_identical_fact_semantics() {
    for platform in [PlatformId::WindowsX64, PlatformId::MacosArm64] {
        let harness = EndToEndHarnessBuilder::for_platform(platform)
            .with_path_native(ToolId::Git, "git version 2.47.1")
            .with_known_native(ToolId::ClaudeCode, "2.1.89 (Claude Code)")
            .without_candidate(ToolId::CodexCli);

        let initial = harness.bootstrap();
        assert!(initial.observations.is_empty());
        assert!(initial.detection_run.is_none());

        let run_id = harness.detect_all().unwrap();
        harness.wait_until_completed(&run_id);
        let final_snapshot = harness.bootstrap();

        assert_eq!(
            facts(&final_snapshot),
            vec![
                (
                    ToolId::Git,
                    ObservedToolState::PresentHealthy,
                    VersionStatus::NotComparable,
                ),
                (
                    ToolId::ClaudeCode,
                    ObservedToolState::PresentPathIssue,
                    VersionStatus::NotComparable,
                ),
                (
                    ToolId::CodexCli,
                    ObservedToolState::Absent,
                    VersionStatus::Unknown
                ),
            ]
        );
        assert_eq!(final_snapshot.last_event_sequence, 5);
        assert_eq!(final_snapshot.snapshot_version, 6);
        assert_eq!(
            final_snapshot.detection_run.unwrap().status,
            DetectionRunStatus::Completed
        );
    }
}

#[test]
fn detection_flow_is_read_only_and_never_requests_privilege_or_network() {
    let harness = EndToEndHarnessBuilder::for_platform(PlatformId::WindowsX64).with_all_absent();
    let run_id = harness.detect_all().unwrap();
    harness.wait_until_completed(&run_id);

    assert!(harness.process_requests().is_empty());
    assert_eq!(harness.privilege_requests(), 0);
    assert_eq!(harness.network_requests(), 0);
}

#[test]
fn process_request_facts_never_include_shell_or_mutating_arguments() {
    let harness = EndToEndHarnessBuilder::for_platform(PlatformId::MacosArm64)
        .with_path_native(ToolId::Git, "git version 2.47.1")
        .with_known_native(ToolId::ClaudeCode, "2.1.89 (Claude Code)")
        .without_candidate(ToolId::CodexCli);
    let run_id = harness.detect_all().unwrap();
    harness.wait_until_completed(&run_id);

    let requests = harness.process_requests();
    assert_eq!(requests.len(), 2);
    assert!(requests.iter().all(|request| {
        request.args == vec!["--version"]
            && request.environment.is_empty()
            && request.executable.is_absolute()
    }));
    assert!(requests.iter().all(|request| {
        !request
            .args
            .iter()
            .any(|argument| argument == "install" || argument == "upgrade")
    }));
    assert!(
        requests
            .iter()
            .all(|request| request.executable.to_string_lossy() != "/bin/sh")
    );
}
