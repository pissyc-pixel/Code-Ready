mod ccswitch;
mod claude;
mod codex;
mod git;
mod node;
mod npm_global;
mod opencode;
mod python;
mod shared;
mod winget;

use serde::{Deserialize, Serialize};
use tauri::Emitter;

use crate::config;

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

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DetectResultEvent {
    pub tool_id: String,
    pub result: ToolStatus,
}

const ALL_TOOL_IDS: [&str; 9] = [
    "winget",
    "git",
    "node",
    "npm",
    "python",
    "claude",
    "codex",
    "opencode",
    "ccswitch",
];

pub async fn detect_tool(
    _app: tauri::AppHandle,
    tool_id: &str,
) -> Result<ToolStatus, String> {
    let result = match tool_id {
        "winget" => winget::detect(),
        "git" => git::detect(),
        "node" => node::detect_node(),
        "npm" => detect_npm(),
        "python" => python::detect(),
        "claude" => claude::detect(),
        "codex" => codex::detect(),
        "opencode" => opencode::detect(),
        "ccswitch" => {
            let configured_path = config::get_config()
                .ok()
                .and_then(|item| item.ccswitch_path.map(std::path::PathBuf::from));
            ccswitch::detect_with_config_path(configured_path)
        }
        _ => {
            return Err(format!("unsupported tool id: {tool_id}"));
        }
    };

    Ok(result)
}

pub async fn detect_all_tools(app: tauri::AppHandle) -> Result<Vec<ToolStatus>, String> {
    let mut results = Vec::with_capacity(ALL_TOOL_IDS.len());
    for tool_id in ALL_TOOL_IDS {
        let result = detect_tool(app.clone(), tool_id).await?;
        app.emit(
            "detect:result",
            DetectResultEvent {
                tool_id: tool_id.to_string(),
                result: result.clone(),
            },
        )
        .map_err(|error| error.to_string())?;
        results.push(result);
    }
    Ok(results)
}

#[allow(dead_code)]
pub(crate) fn has_node_runtime() -> Result<bool, String> {
    shared::where_command("node")
        .map(|paths| !paths.is_empty())
        .map_err(|error| error.to_string())
}

#[allow(dead_code)]
pub(crate) fn npm_global_probe_path(command_name: &str) -> Result<Option<std::path::PathBuf>, String> {
    npm_global::probe_npm_global_command(command_name).map_err(|error| error.to_string())
}

#[allow(dead_code)]
pub(crate) fn recheck_ai_npm_command(command_name: &str, display_name: &str) -> ToolStatus {
    let where_paths = match shared::where_command(command_name) {
        Ok(paths) => paths,
        Err(error) => {
            return shared::build_tool_status(
                command_name,
                display_name,
                ToolCategory::Ai,
                ToolInstallStatus::DetectFailed,
                None,
                None,
                DetectionMethod::NpmGlobalProbe,
                Some(error.to_string()),
                None,
            )
        }
    };

    let path_probe = match npm_global::probe_npm_global_command(command_name) {
        Ok(path) => path,
        Err(error) => {
            return shared::build_tool_status(
                command_name,
                display_name,
                ToolCategory::Ai,
                ToolInstallStatus::DetectFailed,
                None,
                None,
                DetectionMethod::NpmGlobalProbe,
                Some(error.to_string()),
                None,
            )
        }
    };

    let version_result = if !where_paths.is_empty() {
        shared::run_command(command_name, &["--version"]).map(Some)
    } else if let Some(path) = path_probe.as_ref() {
        shared::run_path_command(path, &["--version"]).map(Some)
    } else {
        Ok(None)
    };

    let status = shared::resolve_auth_sensitive_status(
        !where_paths.is_empty(),
        path_probe.is_some(),
        version_result.clone(),
    );
    let output = version_result.ok().flatten();
    let suggestion = if matches!(status, ToolInstallStatus::InstalledButPathMissing) {
        Some("已探测到 npm 全局命令，但当前 PATH 可能未刷新。".to_string())
    } else {
        None
    };

    shared::build_tool_status(
        command_name,
        display_name,
        ToolCategory::Ai,
        status,
        output.as_ref().and_then(shared::extract_version_line),
        where_paths
            .first()
            .cloned()
            .or_else(|| path_probe.as_ref().map(|path| path.display().to_string())),
        DetectionMethod::NpmGlobalProbe,
        output
            .as_ref()
            .filter(|item| item.exit_code != 0)
            .and_then(shared::command_error_message),
        suggestion,
    )
}

#[allow(dead_code)]
pub(crate) fn current_npm_status() -> ToolStatus {
    detect_npm()
}

#[allow(dead_code)]
pub(crate) fn resolve_ccswitch_path(
    configured_path: Option<std::path::PathBuf>,
) -> Option<std::path::PathBuf> {
    ccswitch::resolve_detected_path(configured_path)
}

fn detect_npm() -> ToolStatus {
    let where_paths = match shared::where_command("npm") {
        Ok(paths) => paths,
        Err(error) => {
            return shared::build_tool_status(
                "npm",
                "npm",
                ToolCategory::Base,
                ToolInstallStatus::DetectFailed,
                None,
                None,
                DetectionMethod::Combined,
                Some(error.to_string()),
                None,
            )
        }
    };

    let common_npm = shared::program_files_dir()
        .map(|path| path.join("nodejs").join("npm.cmd"))
        .filter(|path| path.exists());
    let npm_global_probe = match npm_global::probe_npm_global_command("npm") {
        Ok(path) => path,
        Err(error) => {
            return shared::build_tool_status(
                "npm",
                "npm",
                ToolCategory::Base,
                ToolInstallStatus::DetectFailed,
                None,
                None,
                DetectionMethod::NpmGlobalProbe,
                Some(error.to_string()),
                None,
            )
        }
    };
    let path_probe = npm_global_probe.or(common_npm);

    let version_result = if !where_paths.is_empty() {
        shared::run_command("npm", &["-v"]).map(Some)
    } else if let Some(path) = path_probe.as_ref() {
        shared::run_path_command(path, &["-v"]).map(Some)
    } else {
        Ok(None)
    };

    let status = shared::resolve_cli_status(!where_paths.is_empty(), path_probe.is_some(), version_result.clone());
    let output = version_result.ok().flatten();
    let suggestion = if matches!(status, ToolInstallStatus::InstalledButPathMissing) {
        Some("检测到 npm 全局命令，但当前 PATH 可能未刷新。".to_string())
    } else {
        None
    };

    shared::build_tool_status(
        "npm",
        "npm",
        ToolCategory::Base,
        status,
        output.as_ref().and_then(shared::extract_version_line),
        where_paths
            .first()
            .cloned()
            .or_else(|| path_probe.as_ref().map(|path| path.display().to_string())),
        DetectionMethod::NpmGlobalProbe,
        output
            .as_ref()
            .filter(|item| item.exit_code != 0)
            .map(|item| item.stderr.trim().to_string())
            .filter(|message| !message.is_empty()),
        suggestion,
    )
}
