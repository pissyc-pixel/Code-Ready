use super::{
    npm_global,
    shared::{
        build_tool_status, command_error_message, extract_version_line,
        resolve_auth_sensitive_status, run_command, run_path_command, where_command,
    },
    DetectionMethod, ToolCategory, ToolInstallStatus, ToolStatus,
};

pub fn detect() -> ToolStatus {
    detect_auth_sensitive_cli("claude", "Claude Code")
}

fn detect_auth_sensitive_cli(command: &str, name: &str) -> ToolStatus {
    let where_paths = match where_command(command) {
        Ok(paths) => paths,
        Err(error) => {
            return build_tool_status(
                command,
                name,
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

    let path_probe = match npm_global::probe_npm_global_command(command) {
        Ok(path) => path,
        Err(error) => {
            return build_tool_status(
                command,
                name,
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
        run_command(command, &["--version"]).map(Some)
    } else if let Some(path) = path_probe.as_ref() {
        run_path_command(path, &["--version"]).map(Some)
    } else {
        Ok(None)
    };

    let status = resolve_auth_sensitive_status(
        !where_paths.is_empty(),
        path_probe.is_some(),
        version_result.clone(),
    );
    let output = version_result.ok().flatten();
    let needs_config = matches!(
        status.clone(),
        ToolInstallStatus::Installed | ToolInstallStatus::InstalledButPathMissing
    );
    let show_error = matches!(
        status.clone(),
        ToolInstallStatus::Broken | ToolInstallStatus::DetectFailed
    );
    let suggestion = needs_config.then(|| {
        "Detected CLI files, but login or local configuration may still be required."
            .to_string()
    });

    build_tool_status(
        command,
        name,
        ToolCategory::Ai,
        status,
        output.as_ref().and_then(extract_version_line),
        where_paths
            .first()
            .cloned()
            .or_else(|| path_probe.as_ref().map(|path| path.display().to_string())),
        DetectionMethod::NpmGlobalProbe,
        output
            .as_ref()
            .filter(|item| item.exit_code != 0)
            .and_then(command_error_message)
            .filter(|_| show_error),
        suggestion,
    )
}
