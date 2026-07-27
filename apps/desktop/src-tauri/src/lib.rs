use std::sync::Arc;

use application::bootstrap::BootstrapService;
use platform::native::NativePlatformAdapter;

pub mod api;
pub mod application;
pub mod domain;
pub mod platform;
pub mod tools;

pub fn run() -> tauri::Result<()> {
    let adapter = Arc::new(NativePlatformAdapter::new().map_err(|error| {
        let boxed: Box<dyn std::error::Error> = Box::new(error);
        tauri::Error::Setup(boxed.into())
    })?);
    let service = BootstrapService::new(adapter);

    tauri::Builder::default()
        .manage(service)
        .invoke_handler(tauri::generate_handler![api::commands::get_bootstrap_state])
        .run(tauri::generate_context!())
}
