use tauri::State;

use crate::{
    installer::{now_timestamp, runner, InstallPhase, InstallTaskState},
    process::kill_tree::{kill_process_tree, KillTreeOutcome},
    state::AppState,
};

#[tauri::command]
pub async fn install_tool(
    app: tauri::AppHandle,
    state: State<'_, AppState>,
    tool_id: String,
) -> Result<(), String> {
    let spec = runner::build_command_spec(&tool_id)?;
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
    let snapshot = state
        .snapshot_install()?
        .ok_or_else(|| "no running install task".to_string())?;

    match snapshot.pid {
        Some(pid) => match kill_process_tree(pid)? {
            KillTreeOutcome::Killed => state.mark_cancel_requested(&snapshot.tool_id)?,
            KillTreeOutcome::AlreadyExited => {}
        },
        None => state.mark_cancel_requested(&snapshot.tool_id)?,
    }
    Ok(())
}
