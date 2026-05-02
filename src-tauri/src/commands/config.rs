use crate::config::{self, AppConfig, AppConfigPatch};

#[tauri::command]
pub async fn get_config() -> Result<AppConfig, String> {
    config::get_config()
}

#[tauri::command]
pub async fn update_config(patch: AppConfigPatch) -> Result<AppConfig, String> {
    config::update_config(patch)
}

#[tauri::command]
pub async fn reset_config() -> Result<AppConfig, String> {
    config::reset_config()
}
