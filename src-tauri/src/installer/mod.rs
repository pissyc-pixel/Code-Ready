pub mod claude;
pub mod ccswitch;
pub mod codex;
pub mod git;
pub mod node;
pub mod npm;
pub mod opencode;
pub mod python;
pub mod runner;

use serde::Serialize;

use crate::detector::ToolInstallStatus;

#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum InstallPhase {
    Started,
    Running,
    Success,
    Failed,
    Cancelled,
    Timeout,
}

#[allow(dead_code)]
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct InstallResult {
    pub exit_code: Option<i32>,
    pub duration_ms: Option<u64>,
}

#[allow(dead_code)]
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct InstallProgressEvent {
    pub tool_id: String,
    pub phase: InstallPhase,
    pub line: String,
    pub stream: String,
    pub timestamp: String,
}

#[allow(dead_code)]
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct InstallStatusEvent {
    pub tool_id: String,
    pub status: ToolInstallStatus,
    pub phase: InstallPhase,
    pub error_message: Option<String>,
    pub suggestion: Option<String>,
    pub result: Option<InstallResult>,
    pub timestamp: String,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct InstallTaskState {
    pub tool_id: String,
    pub pid: Option<u32>,
    pub phase: InstallPhase,
    pub started_at: String,
    pub cancel_requested: bool,
}

pub fn now_timestamp() -> String {
    chrono::Local::now().to_rfc3339()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn serializes_install_status_event() {
        let event = InstallStatusEvent {
            tool_id: "git".to_string(),
            status: ToolInstallStatus::Installing,
            phase: InstallPhase::Started,
            error_message: None,
            suggestion: None,
            result: None,
            timestamp: "2026-05-02T18:00:00+08:00".to_string(),
        };

        let json = serde_json::to_value(event).expect("serialize event");
        assert_eq!(json["toolId"], "git");
        assert_eq!(json["status"], "installing");
        assert_eq!(json["phase"], "started");
    }
}
