mod auth_cli;
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

use std::path::PathBuf;

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
pub(crate) fn npm_global_probe_path(
    command_name: &str,
) -> Result<Option<std::path::PathBuf>, String> {
    npm_global::probe_npm_global_command(command_name).map_err(|error| error.to_string())
}

#[allow(dead_code)]
pub(crate) fn recheck_ai_npm_command(command_name: &str, display_name: &str) -> ToolStatus {
    auth_cli::detect_auth_sensitive_cli(command_name, display_name)
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
    let where_npm_cmd_paths = match shared::where_command("npm.cmd") {
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

    let where_npm_paths = match shared::where_command("npm") {
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

    let npm_global_probe = npm_global::probe_npm_global_command("npm").unwrap_or(None);
    let candidates = npm_probe_candidates(
        &where_npm_cmd_paths,
        &where_npm_paths,
        shared::program_files_dir(),
        shared::appdata_dir(),
        npm_global_probe,
    );
    let path_probe = shared::first_existing_path(&candidates);
    let where_hit = !where_npm_cmd_paths.is_empty() || !where_npm_paths.is_empty();
    let version_result = if let Some(path) = path_probe.as_ref() {
        shared::run_path_command(path, &["-v"]).map(Some)
    } else {
        Ok(None)
    };
    let node_on_path = has_node_runtime().unwrap_or(false);
    let missing_error_message = if node_on_path && path_probe.is_none() {
        Some(
            "Node.js is installed, but npm was not found in PATH or common install locations."
                .to_string(),
        )
    } else {
        None
    };

    build_npm_status(
        where_hit,
        path_probe,
        version_result,
        node_on_path,
        missing_error_message,
    )
}

fn npm_probe_candidates(
    where_npm_cmd_paths: &[String],
    where_npm_paths: &[String],
    program_files_dir: Option<PathBuf>,
    appdata_dir: Option<PathBuf>,
    npm_global_probe: Option<PathBuf>,
) -> Vec<PathBuf> {
    let mut candidates = Vec::new();

    for candidate in where_npm_cmd_paths
        .iter()
        .chain(where_npm_paths.iter())
        .map(PathBuf::from)
    {
        push_unique_path(&mut candidates, candidate);
    }

    if let Some(program_files_dir) = program_files_dir {
        push_unique_path(
            &mut candidates,
            program_files_dir.join("nodejs").join("npm.cmd"),
        );
    }

    if let Some(appdata_dir) = appdata_dir {
        push_unique_path(&mut candidates, appdata_dir.join("npm").join("npm.cmd"));
    }

    if let Some(npm_global_probe) = npm_global_probe {
        push_unique_path(&mut candidates, npm_global_probe);
    }

    candidates
}

fn push_unique_path(paths: &mut Vec<PathBuf>, candidate: PathBuf) {
    if !paths.iter().any(|existing| existing == &candidate) {
        paths.push(candidate);
    }
}

fn build_npm_status(
    where_hit: bool,
    path_probe: Option<PathBuf>,
    version_result: Result<Option<shared::CommandOutput>, shared::ProbeError>,
    node_on_path: bool,
    missing_error_message: Option<String>,
) -> ToolStatus {
    let status = shared::resolve_cli_status(where_hit, path_probe.is_some(), version_result.clone());
    let output = version_result.clone().ok().flatten();
    let detection_method = if where_hit {
        DetectionMethod::Which
    } else if path_probe.is_some() {
        DetectionMethod::PathProbe
    } else {
        DetectionMethod::Combined
    };
    let suggestion = if matches!(status, ToolInstallStatus::InstalledButPathMissing) {
        Some("Detected npm on disk, but the current PATH may not include it yet.".to_string())
    } else if matches!(status, ToolInstallStatus::Missing) && node_on_path {
        Some(
            "Node.js was detected. Reinstall Node.js or repair npm manually; this app will not change PATH automatically."
                .to_string(),
        )
    } else {
        None
    };
    let error_message = match (&version_result, &status) {
        (Err(error), _) => Some(error.to_string()),
        (Ok(Some(output)), ToolInstallStatus::Broken | ToolInstallStatus::DetectFailed) => {
            shared::command_error_message(output)
        }
        (Ok(None), ToolInstallStatus::Missing) => missing_error_message,
        _ => None,
    };

    shared::build_tool_status(
        "npm",
        "npm",
        ToolCategory::Base,
        status,
        output.as_ref().and_then(shared::extract_version_line),
        path_probe.as_ref().map(|path| path.display().to_string()),
        detection_method,
        error_message,
        suggestion,
    )
}

#[cfg(test)]
mod tests {
    use super::{build_npm_status, npm_probe_candidates, DetectionMethod, ToolInstallStatus};
    use crate::detector::shared::CommandOutput;
    use std::path::PathBuf;

    #[test]
    fn npm_probe_candidates_prefer_npm_cmd_and_program_files_path() {
        let candidates = npm_probe_candidates(
            &[r"C:\Users\Tester\AppData\Roaming\npm\npm.cmd".to_string()],
            &[r"C:\Users\Tester\AppData\Roaming\npm\npm".to_string()],
            Some(PathBuf::from(r"C:\Program Files")),
            Some(PathBuf::from(r"C:\Users\Tester\AppData\Roaming")),
            Some(PathBuf::from(r"D:\extra\npm.cmd")),
        );

        assert_eq!(
            candidates[0],
            PathBuf::from(r"C:\Users\Tester\AppData\Roaming\npm\npm.cmd")
        );
        assert!(candidates.contains(&PathBuf::from(
            r"C:\Program Files\nodejs\npm.cmd"
        )));
    }

    #[test]
    fn npm_status_is_not_detect_failed_when_node_exists_but_npm_prefix_probe_fails() {
        let status = build_npm_status(
            false,
            None,
            Ok(None),
            true,
            Some(
                "Node.js is installed, but npm was not found in PATH or common install locations."
                    .to_string(),
            ),
        );

        assert!(!matches!(status.status, ToolInstallStatus::DetectFailed));
        assert!(matches!(status.status, ToolInstallStatus::Missing));
        assert!(status
            .error_message
            .as_deref()
            .unwrap_or_default()
            .contains("Node.js"));
    }

    #[test]
    fn npm_status_marks_path_missing_when_specific_npm_cmd_path_works_without_where_hit() {
        let status = build_npm_status(
            false,
            Some(PathBuf::from(r"C:\Program Files\nodejs\npm.cmd")),
            Ok(Some(CommandOutput {
                exit_code: 0,
                stdout: "10.8.0".to_string(),
                stderr: String::new(),
            })),
            false,
            None,
        );

        assert!(matches!(
            status.status,
            ToolInstallStatus::InstalledButPathMissing
        ));
        assert!(matches!(status.detection_method, DetectionMethod::PathProbe));
    }
}
