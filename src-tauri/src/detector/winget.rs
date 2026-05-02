use std::path::PathBuf;

use super::{
    shared::{
        build_tool_status, extract_version_line, program_files_dir, resolve_cli_status,
        run_command, run_path_command, where_command,
    },
    DetectionMethod, ToolCategory, ToolStatus,
};

pub fn detect() -> ToolStatus {
    let where_paths = match where_command("winget") {
        Ok(paths) => paths,
        Err(error) => {
            return build_tool_status(
                "winget",
                "winget",
                ToolCategory::Base,
                super::ToolInstallStatus::DetectFailed,
                None,
                None,
                DetectionMethod::Combined,
                Some(error.to_string()),
                None,
            )
        }
    };

    let path_probe = first_winget_path();
    let version_result = if let Some(path) = path_probe.as_ref().filter(|_| where_paths.is_empty()) {
        run_path_command(path, &["--version"]).map(Some)
    } else {
        run_command("winget", &["--version"]).map(Some)
    };

    let status = resolve_cli_status(!where_paths.is_empty(), path_probe.is_some(), version_result.clone());
    let output = version_result.ok().flatten();
    let suggestion = if matches!(status, super::ToolInstallStatus::Installed) {
        if let Ok(source_output) = run_command("winget", &["source", "list"]) {
            let lower = format!("{}\n{}", source_output.stdout, source_output.stderr).to_ascii_lowercase();
            if lower.contains("agreement") || lower.contains("terms") {
                Some("检测到 winget source agreement 提示，首次使用可能需要手动确认。".to_string())
            } else {
                None
            }
        } else {
            None
        }
    } else {
        None
    };

    build_tool_status(
        "winget",
        "winget",
        ToolCategory::Base,
        status,
        output.as_ref().and_then(extract_version_line),
        where_paths
            .first()
            .cloned()
            .or_else(|| path_probe.as_ref().map(|path| path.display().to_string())),
        DetectionMethod::Combined,
        output
            .as_ref()
            .filter(|item| item.exit_code != 0)
            .map(|item| item.stderr.trim().to_string())
            .filter(|message| !message.is_empty()),
        suggestion,
    )
}

fn first_winget_path() -> Option<PathBuf> {
    std::env::var_os("LOCALAPPDATA")
        .map(PathBuf::from)
        .map(|path| path.join("Microsoft").join("WindowsApps").join("winget.exe"))
        .filter(|path| path.exists())
        .or_else(|| {
            program_files_dir()
                .map(|path| path.join("WindowsApps").join("winget.exe"))
                .filter(|path| path.exists())
        })
}
