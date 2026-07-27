use std::sync::Arc;

use api::events::TauriDetectionEventSink;
use application::bootstrap::BootstrapService;
use application::detection::DetectionService;
use application::snapshot_store::SnapshotStore;
use domain::tool_registry::built_in_tool_registry;
use platform::PlatformAdapter;
use platform::native::NativePlatformAdapter;
use platform::process::{NativeProcessRunner, SystemClock};
use tauri::Manager;
use tools::built_in_detector_registry;

pub mod api;
pub mod application;
pub mod domain;
pub mod platform;
pub mod tools;

pub fn run() -> tauri::Result<()> {
    tauri::Builder::default()
        .setup(|app| {
            let platform = Arc::new(NativePlatformAdapter::new()?);
            let clock = Arc::new(SystemClock);
            let store = Arc::new(SnapshotStore::new(
                platform.platform(),
                built_in_tool_registry(),
                clock.clone(),
            ));
            let bootstrap = BootstrapService::new(store.clone());
            let event_sink = Arc::new(TauriDetectionEventSink::new(app.handle().clone()));
            let detection = DetectionService::new(
                store,
                platform,
                Arc::new(NativeProcessRunner),
                clock,
                built_in_detector_registry(),
                event_sink,
            );

            app.manage(bootstrap);
            app.manage(detection);
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            api::commands::bootstrap,
            api::commands::detect_tools
        ])
        .run(tauri::generate_context!())
}
