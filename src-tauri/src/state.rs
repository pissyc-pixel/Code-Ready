use std::sync::Mutex;

use crate::installer::InstallTaskState;

#[derive(Default)]
pub struct AppState {
    pub current_install: Mutex<Option<InstallTaskState>>,
}
