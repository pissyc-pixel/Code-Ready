use std::{
    fmt,
    path::{Path, PathBuf},
    process::Stdio,
};

use chrono::Utc;

use super::{DetectionMethod, ToolCategory, ToolInstallStatus, ToolStatus};
use crate::process::command::new_command;

#[derive(Debug, Clone)]
pub struct CommandOutput {
    pub exit_code: i32,
    pub stdout: String,
    pub stderr: String,
}

#[derive(Debug, Clone)]
pub enum ProbeError {
    Spawn(String),
}

impl fmt::Display for ProbeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Spawn(message) => write!(f, "{message}"),
        }
    }
}

pub fn now_iso_string() -> String {
    Utc::now().to_rfc3339()
}

pub fn resolve_cli_status(
    where_hit: bool,
    path_hit: bool,
    version_result: Result<Option<CommandOutput>, ProbeError>,
) -> ToolInstallStatus {
    match version_result {
        Err(_) => ToolInstallStatus::DetectFailed,
        Ok(Some(output)) if output.exit_code == 0 && where_hit => ToolInstallStatus::Installed,
        Ok(Some(output)) if output.exit_code == 0 && path_hit => {
            ToolInstallStatus::InstalledButPathMissing
        }
        Ok(Some(_)) if where_hit || path_hit => ToolInstallStatus::Broken,
        Ok(Some(output)) if output.exit_code == 0 => ToolInstallStatus::Installed,
        Ok(Some(_)) => ToolInstallStatus::Broken,
        Ok(None) if path_hit => ToolInstallStatus::InstalledButPathMissing,
        Ok(None) if where_hit => ToolInstallStatus::Broken,
        Ok(None) => ToolInstallStatus::Missing,
    }
}

pub fn resolve_auth_sensitive_status(
    where_hit: bool,
    path_hit: bool,
    version_result: Result<Option<CommandOutput>, ProbeError>,
) -> ToolInstallStatus {
    match version_result {
        Err(_) => ToolInstallStatus::DetectFailed,
        Ok(Some(output)) if output.exit_code == 0 && where_hit => ToolInstallStatus::Installed,
        Ok(Some(output)) if output.exit_code == 0 && path_hit => {
            ToolInstallStatus::InstalledButPathMissing
        }
        Ok(Some(output)) if is_auth_or_config_required(&output) && where_hit => {
            ToolInstallStatus::Installed
        }
        Ok(Some(output)) if is_auth_or_config_required(&output) && path_hit => {
            ToolInstallStatus::InstalledButPathMissing
        }
        Ok(Some(_)) if where_hit || path_hit => ToolInstallStatus::Broken,
        Ok(Some(output)) if output.exit_code == 0 => ToolInstallStatus::Installed,
        Ok(Some(_)) => ToolInstallStatus::Broken,
        Ok(None) if path_hit => ToolInstallStatus::InstalledButPathMissing,
        Ok(None) if where_hit => ToolInstallStatus::Broken,
        Ok(None) => ToolInstallStatus::Missing,
    }
}

pub fn build_tool_status(
    id: &str,
    name: &str,
    category: ToolCategory,
    status: ToolInstallStatus,
    version: Option<String>,
    executable_path: Option<String>,
    detection_method: DetectionMethod,
    error_message: Option<String>,
    suggestion: Option<String>,
) -> ToolStatus {
    ToolStatus {
        id: id.to_string(),
        name: name.to_string(),
        category,
        status,
        version,
        executable_path,
        detection_method,
        error_message,
        suggestion,
        last_checked_at: now_iso_string(),
    }
}

pub fn extract_version_line(output: &CommandOutput) -> Option<String> {
    output
        .stdout
        .lines()
        .chain(output.stderr.lines())
        .map(str::trim)
        .find(|line| !line.is_empty())
        .map(ToOwned::to_owned)
}

pub fn command_error_message(output: &CommandOutput) -> Option<String> {
    let stderr = output.stderr.trim();
    if !stderr.is_empty() {
        return Some(stderr.to_string());
    }

    let stdout = output.stdout.trim();
    if !stdout.is_empty() {
        return Some(stdout.to_string());
    }

    None
}

pub fn is_auth_or_config_required(output: &CommandOutput) -> bool {
    let combined = format!("{}\n{}", output.stdout, output.stderr).to_ascii_lowercase();
    [
        "login",
        "log in",
        "auth",
        "api key",
        "not authenticated",
        "please sign in",
        "sign in",
        "signin",
        "unauthorized",
    ]
    .iter()
    .any(|keyword| combined.contains(keyword))
}

pub fn run_command(program: &str, args: &[&str]) -> Result<CommandOutput, ProbeError> {
    let output = new_command(program)
        .args(args)
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output()
        .map_err(|error| ProbeError::Spawn(error.to_string()))?;

    Ok(CommandOutput {
        exit_code: output.status.code().unwrap_or(-1),
        stdout: String::from_utf8_lossy(&output.stdout).into_owned(),
        stderr: String::from_utf8_lossy(&output.stderr).into_owned(),
    })
}

pub fn run_path_command(path: &Path, args: &[&str]) -> Result<CommandOutput, ProbeError> {
    let extension = path
        .extension()
        .and_then(|ext| ext.to_str())
        .unwrap_or_default()
        .to_ascii_lowercase();

    if extension == "cmd" || extension == "bat" {
        let mut cmd_args = vec!["/c".to_string(), path.display().to_string()];
        cmd_args.extend(args.iter().map(|arg| arg.to_string()));
        return run_command_owned("cmd", &cmd_args);
    }

    if extension == "ps1" {
        let mut ps_args = vec![
            "-NoProfile".to_string(),
            "-ExecutionPolicy".to_string(),
            "Bypass".to_string(),
            "-File".to_string(),
            path.display().to_string(),
        ];
        ps_args.extend(args.iter().map(|arg| arg.to_string()));
        return run_command_owned("powershell", &ps_args);
    }

    run_command_owned(path.display().to_string(), &args.iter().map(|arg| arg.to_string()).collect::<Vec<_>>())
}

fn run_command_owned(program: impl AsRef<str>, args: &[String]) -> Result<CommandOutput, ProbeError> {
    let output = new_command(program.as_ref())
        .args(args)
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output()
        .map_err(|error| ProbeError::Spawn(error.to_string()))?;

    Ok(CommandOutput {
        exit_code: output.status.code().unwrap_or(-1),
        stdout: String::from_utf8_lossy(&output.stdout).into_owned(),
        stderr: String::from_utf8_lossy(&output.stderr).into_owned(),
    })
}

pub fn where_command(command: &str) -> Result<Vec<String>, ProbeError> {
    let output = run_command("where.exe", &[command])?;
    if output.exit_code != 0 {
        return Ok(Vec::new());
    }

    Ok(output
        .stdout
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty())
        .map(ToOwned::to_owned)
        .collect())
}

pub fn first_existing_path(paths: &[PathBuf]) -> Option<PathBuf> {
    paths.iter().find(|path| path.exists()).cloned()
}

pub fn appdata_dir() -> Option<PathBuf> {
    std::env::var_os("APPDATA").map(PathBuf::from)
}

pub fn localappdata_dir() -> Option<PathBuf> {
    std::env::var_os("LOCALAPPDATA").map(PathBuf::from)
}

pub fn program_files_dir() -> Option<PathBuf> {
    std::env::var_os("ProgramFiles").map(PathBuf::from)
}

pub fn user_profile_dir() -> Option<PathBuf> {
    std::env::var_os("USERPROFILE").map(PathBuf::from)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ok_output() -> CommandOutput {
        CommandOutput {
            exit_code: 0,
            stdout: "v1.0.0".to_string(),
            stderr: String::new(),
        }
    }

    fn bad_output() -> CommandOutput {
        CommandOutput {
            exit_code: 1,
            stdout: String::new(),
            stderr: "boom".to_string(),
        }
    }

    #[test]
    fn resolves_installed_when_where_and_version_succeed() {
        let status = resolve_cli_status(true, false, Ok(Some(ok_output())));
        assert!(matches!(status, ToolInstallStatus::Installed));
    }

    #[test]
    fn resolves_path_missing_when_only_path_probe_succeeds() {
        let status = resolve_cli_status(false, true, Ok(Some(ok_output())));
        assert!(matches!(status, ToolInstallStatus::InstalledButPathMissing));
    }

    #[test]
    fn resolves_broken_when_where_hits_but_version_fails() {
        let status = resolve_cli_status(true, false, Ok(Some(bad_output())));
        assert!(matches!(status, ToolInstallStatus::Broken));
    }

    #[test]
    fn resolves_missing_when_nothing_is_found() {
        let status = resolve_cli_status(false, false, Ok(None));
        assert!(matches!(status, ToolInstallStatus::Missing));
    }

    #[test]
    fn resolves_detect_failed_when_probe_errors() {
        let status = resolve_cli_status(
            false,
            false,
            Err(ProbeError::Spawn("spawn failed".to_string())),
        );
        assert!(matches!(status, ToolInstallStatus::DetectFailed));
    }

    #[test]
    fn auth_sensitive_status_treats_login_required_as_installed() {
        let output = CommandOutput {
            exit_code: 1,
            stdout: String::new(),
            stderr: "please sign in before continuing".to_string(),
        };

        let status = resolve_auth_sensitive_status(true, false, Ok(Some(output)));
        assert!(matches!(status, ToolInstallStatus::Installed));
    }

    #[test]
    fn auth_sensitive_status_treats_login_required_path_probe_as_path_missing() {
        let output = CommandOutput {
            exit_code: 1,
            stdout: String::new(),
            stderr: "not authenticated".to_string(),
        };

        let status = resolve_auth_sensitive_status(false, true, Ok(Some(output)));
        assert!(matches!(status, ToolInstallStatus::InstalledButPathMissing));
    }
}
