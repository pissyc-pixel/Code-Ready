use tauri::State;

use crate::{
    config,
    installer::{now_timestamp, runner, InstallPhase, InstallRequestMode, InstallTaskState},
    process::kill_tree::kill_process_tree,
    state::AppState,
};

#[tauri::command]
pub async fn install_tool(
    app: tauri::AppHandle,
    state: State<'_, AppState>,
    tool_id: String,
) -> Result<(), String> {
    start_install_task(app, state, tool_id, InstallRequestMode::Install).await
}

#[tauri::command]
pub async fn reinstall_tool(
    app: tauri::AppHandle,
    state: State<'_, AppState>,
    tool_id: String,
) -> Result<(), String> {
    start_install_task(app, state, tool_id, InstallRequestMode::Reinstall).await
}

#[tauri::command]
pub async fn install_latest_tool(
    app: tauri::AppHandle,
    state: State<'_, AppState>,
    tool_id: String,
) -> Result<(), String> {
    start_install_task(app, state, tool_id, InstallRequestMode::Latest).await
}

async fn start_install_task(
    app: tauri::AppHandle,
    state: State<'_, AppState>,
    tool_id: String,
    mode: InstallRequestMode,
) -> Result<(), String> {
    let config = config::get_config()?;
    let spec = runner::build_command_spec_for_mode(&tool_id, &config, mode)?;
    state.reserve_install(InstallTaskState {
        tool_id: tool_id.clone(),
        pid: None,
        phase: InstallPhase::Started,
        started_at: now_timestamp(),
        cancel_requested: false,
    })?;
    runner::spawn_install_runner(app, tool_id, spec);
    Ok(())
}

#[tauri::command]
pub async fn cancel_install(
    _app: tauri::AppHandle,
    state: State<'_, AppState>,
) -> Result<(), String> {
    let snapshot = match state.snapshot_install()? {
        Some(task) => task,
        // Task already finished between UI click and this command — treat as no-op.
        None => return Ok(()),
    };

    // Mark cancel intent.  If the task finished between snapshot and here, silently ignore
    // the resulting error rather than surfacing it to the frontend.
    let _ = state.mark_cancel_requested(&snapshot.tool_id);

    if let Some(pid) = snapshot.pid {
        // Best-effort kill: AlreadyExited and unexpected errors are both acceptable.
        // The runner loop will observe the natural exit and cancel_requested drives
        // the final Cancelled status.
        let _ = kill_process_tree(pid);
    }
    Ok(())
}
