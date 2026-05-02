use tauri::State;

use crate::{installer::InstallTaskState, state::AppState};

#[tauri::command]
pub async fn install_tool(
    _app: tauri::AppHandle,
    state: State<'_, AppState>,
    tool_id: String,
) -> Result<(), String> {
    let mut current_install = state
        .current_install
        .lock()
        .map_err(|_| "install task state poisoned".to_string())?;

    if current_install.is_some() {
        return Err("another install task is already running".to_string());
    }

    *current_install = Some(InstallTaskState { tool_id });
    Ok(())
}

#[tauri::command]
pub async fn cancel_install(
    _app: tauri::AppHandle,
    state: State<'_, AppState>,
) -> Result<(), String> {
    let mut current_install = state
        .current_install
        .lock()
        .map_err(|_| "install task state poisoned".to_string())?;

    if current_install.is_none() {
        return Err("no running install task".to_string());
    }

    *current_install = None;
    Ok(())
}
