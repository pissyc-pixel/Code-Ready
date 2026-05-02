use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ToolCategory {
    Base,
    Ai,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ToolInstallStatus {
    Checking,
    Installed,
    Missing,
    Installing,
    InstallFailed,
    DetectFailed,
    InstalledButPathMissing,
    Broken,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DetectionMethod {
    Which,
    VersionFlag,
    PathProbe,
    Registry,
    NpmGlobalProbe,
    Combined,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ToolStatus {
    pub id: String,
    pub name: String,
    pub category: ToolCategory,
    pub status: ToolInstallStatus,
    pub version: Option<String>,
    pub executable_path: Option<String>,
    pub detection_method: DetectionMethod,
    pub error_message: Option<String>,
    pub suggestion: Option<String>,
    pub last_checked_at: String,
}

pub async fn detect_tool(
    _app: tauri::AppHandle,
    tool_id: &str,
) -> Result<ToolStatus, String> {
    Ok(ToolStatus {
        id: tool_id.to_string(),
        name: tool_id.to_string(),
        category: ToolCategory::Base,
        status: ToolInstallStatus::DetectFailed,
        version: None,
        executable_path: None,
        detection_method: DetectionMethod::Combined,
        error_message: Some("V0.2 skeleton only: detector not implemented yet".to_string()),
        suggestion: None,
        last_checked_at: "1970-01-01T00:00:00Z".to_string(),
    })
}

pub async fn detect_all_tools(_app: tauri::AppHandle) -> Result<Vec<ToolStatus>, String> {
    Ok(Vec::new())
}
