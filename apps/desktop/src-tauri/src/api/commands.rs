use crate::application::bootstrap::BootstrapService;
use crate::application::detection::{DetectionService, StartDetectionError};
use crate::domain::contracts::{AppSnapshot, CommandError, CommandErrorCode, ToolId};

pub(crate) fn bootstrap_inner(service: &BootstrapService) -> AppSnapshot {
    service.get_snapshot()
}

#[tauri::command]
pub fn bootstrap(service: tauri::State<'_, BootstrapService>) -> AppSnapshot {
    bootstrap_inner(&service)
}

pub(crate) fn detect_tools_inner(
    service: &DetectionService,
    tool_ids: Option<Vec<ToolId>>,
) -> Result<String, CommandError> {
    service.start(tool_ids).map_err(command_error)
}

#[tauri::command]
pub fn detect_tools(
    tool_ids: Option<Vec<ToolId>>,
    service: tauri::State<'_, DetectionService>,
) -> Result<String, CommandError> {
    detect_tools_inner(&service, tool_ids)
}

fn command_error(error: StartDetectionError) -> CommandError {
    let (code, retryable) = match error {
        StartDetectionError::InvalidSelection(_) => (CommandErrorCode::InvalidToolSelection, false),
        StartDetectionError::AlreadyRunning(_) => (CommandErrorCode::DetectionAlreadyRunning, true),
        StartDetectionError::Internal => (CommandErrorCode::Internal, true),
    };
    CommandError { code, retryable }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::application::detection::{
        DetectionEventSink, EventPublishError, ThreadSpawnError, ThreadSpawner,
    };
    use crate::application::snapshot_store::SnapshotStore;
    use crate::domain::contracts::PlatformId;
    use crate::domain::tool_registry::built_in_tool_registry;
    use crate::platform::PlatformAdapter;
    use crate::platform::fake::FakePlatformAdapter;
    use crate::platform::fake_process::FakeProcessRunner;
    use crate::platform::process::ProcessOutcome;
    use crate::tools::built_in_detector_registry;
    use crate::tools::shared::Clock;
    use std::sync::Arc;

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
            _event: crate::domain::contracts::DetectionEventEnvelope,
        ) -> Result<(), EventPublishError> {
            Ok(())
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

    struct FakeAppServices {
        bootstrap: BootstrapService,
        detection: DetectionService,
    }

    fn fake_app_services(platform: PlatformId) -> FakeAppServices {
        let clock: Arc<dyn Clock> = Arc::new(FixedClock);
        let store = Arc::new(SnapshotStore::new(
            platform.clone(),
            built_in_tool_registry(),
            clock.clone(),
        ));
        let bootstrap = BootstrapService::new(store.clone());
        let platform: Arc<dyn PlatformAdapter> = Arc::new(FakePlatformAdapter::new(platform));
        let runner = Arc::new(FakeProcessRunner::new(std::iter::empty::<ProcessOutcome>()));
        let detection = DetectionService::with_thread_spawner(
            store,
            platform,
            runner,
            clock,
            built_in_detector_registry(),
            Arc::new(NoopEventSink),
            Arc::new(NoopThreadSpawner),
        );
        FakeAppServices {
            bootstrap,
            detection,
        }
    }

    #[test]
    fn bootstrap_core_returns_the_current_snapshot() {
        let services = fake_app_services(PlatformId::WindowsX64);
        let snapshot = bootstrap_inner(&services.bootstrap);

        assert_eq!(snapshot.platform, PlatformId::WindowsX64);
        assert_eq!(snapshot.schema_version, 2);
        assert_eq!(snapshot.snapshot_version, 1);
    }

    #[test]
    fn detect_tools_core_accepts_full_or_subset_and_rejects_slice_two_tools() {
        let services = fake_app_services(PlatformId::MacosArm64);
        assert!(detect_tools_inner(&services.detection, None).is_ok());

        let other = fake_app_services(PlatformId::MacosArm64);
        let error = detect_tools_inner(&other.detection, Some(vec![ToolId::Nodejs]))
            .expect_err("Slice 2 tool must be rejected");
        assert_eq!(error.code, CommandErrorCode::InvalidToolSelection);
        assert!(!error.retryable);
    }
}
