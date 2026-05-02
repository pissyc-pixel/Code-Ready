mod git;
mod node;
mod npm_global;
mod python;
mod shared;
mod winget;

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
        "claude" | "codex" | "opencode" | "ccswitch" => shared::build_tool_status(
            tool_id,
            tool_id,
            ToolCategory::Ai,
            ToolInstallStatus::DetectFailed,
            None,
            None,
            DetectionMethod::Combined,
            Some("V0.2 phase 2: AI detector not implemented yet".to_string()),
            None,
        ),
        _ => {
            return Err(format!("unsupported tool id: {tool_id}"));
        }
    };

    Ok(result)
}

pub async fn detect_all_tools(app: tauri::AppHandle) -> Result<Vec<ToolStatus>, String> {
    let mut results = Vec::with_capacity(ALL_TOOL_IDS.len());
    for tool_id in ALL_TOOL_IDS {
        results.push(detect_tool(app.clone(), tool_id).await?);
    }
    Ok(results)
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
                Some(format!("{error:?}")),
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
                Some(format!("{error:?}")),
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
