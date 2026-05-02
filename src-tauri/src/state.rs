use std::sync::Mutex;

use crate::installer::InstallTaskState;

#[derive(Default)]
pub struct AppState {
    pub current_install: Mutex<Option<InstallTaskState>>,
}

impl AppState {
    pub fn reserve_install(&self, task: InstallTaskState) -> Result<(), String> {
        let mut current_install = self
            .current_install
            .lock()
            .map_err(|_| "install task state poisoned".to_string())?;

        if current_install.is_some() {
            return Err("another install task is already running".to_string());
        }

        *current_install = Some(task);
        Ok(())
    }

    pub fn update_install_pid(&self, tool_id: &str, pid: u32) -> Result<(), String> {
        let mut current_install = self
            .current_install
            .lock()
            .map_err(|_| "install task state poisoned".to_string())?;

        let task = current_install
            .as_mut()
            .ok_or_else(|| "no running install task".to_string())?;

        if task.tool_id != tool_id {
            return Err("install task changed while starting".to_string());
        }

        task.pid = Some(pid);
        Ok(())
    }

    pub fn mark_cancel_requested(&self, tool_id: &str) -> Result<(), String> {
        let mut current_install = self
            .current_install
            .lock()
            .map_err(|_| "install task state poisoned".to_string())?;

        let task = current_install
            .as_mut()
            .ok_or_else(|| "no running install task".to_string())?;

        if task.tool_id != tool_id {
            return Err("install task changed while cancelling".to_string());
        }

        task.cancel_requested = true;
        Ok(())
    }

    pub fn snapshot_install(&self) -> Result<Option<InstallTaskState>, String> {
        let current_install = self
            .current_install
            .lock()
            .map_err(|_| "install task state poisoned".to_string())?;

        Ok(current_install.clone())
    }

    pub fn is_cancel_requested(&self, tool_id: &str) -> Result<bool, String> {
        let current_install = self
            .current_install
            .lock()
            .map_err(|_| "install task state poisoned".to_string())?;

        Ok(current_install
            .as_ref()
            .filter(|task| task.tool_id == tool_id)
            .map(|task| task.cancel_requested)
            .unwrap_or(false))
    }

    pub fn clear_install(&self, tool_id: &str) -> Result<(), String> {
        let mut current_install = self
            .current_install
            .lock()
            .map_err(|_| "install task state poisoned".to_string())?;

        if matches!(current_install.as_ref(), Some(task) if task.tool_id == tool_id) {
            *current_install = None;
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::AppState;
    use crate::installer::{InstallPhase, InstallTaskState};

    #[test]
    fn reserve_install_rejects_second_task() {
        let state = AppState::default();
        state
            .reserve_install(InstallTaskState {
                tool_id: "git".to_string(),
                pid: None,
                phase: InstallPhase::Started,
                started_at: "2026-05-02T18:00:00+08:00".to_string(),
                cancel_requested: false,
            })
            .expect("first task should reserve");

        let error = state
            .reserve_install(InstallTaskState {
                tool_id: "python".to_string(),
                pid: None,
                phase: InstallPhase::Started,
                started_at: "2026-05-02T18:01:00+08:00".to_string(),
                cancel_requested: false,
            })
            .expect_err("second task should fail");

        assert!(error.contains("already running"));
    }

    #[test]
    fn clear_install_releases_lock() {
        let state = AppState::default();
        state
            .reserve_install(InstallTaskState {
                tool_id: "git".to_string(),
                pid: None,
                phase: InstallPhase::Started,
                started_at: "2026-05-02T18:00:00+08:00".to_string(),
                cancel_requested: false,
            })
            .expect("reserve task");

        state.clear_install("git").expect("clear task");
        assert!(state.snapshot_install().expect("snapshot").is_none());
    }
}
