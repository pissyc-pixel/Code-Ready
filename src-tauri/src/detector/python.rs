use std::path::PathBuf;

use super::{
    shared::{
        build_tool_status, extract_version_line, first_existing_path, program_files_dir,
        resolve_cli_status, run_command, run_path_command, user_profile_dir, where_command,
    },
    DetectionMethod, ToolCategory, ToolStatus,
};

pub fn detect() -> ToolStatus {
    let python_paths = match where_command("python") {
        Ok(paths) => paths,
        Err(error) => {
            return build_tool_status(
                "python",
                "Python",
                ToolCategory::Base,
                super::ToolInstallStatus::DetectFailed,
                None,
                None,
                DetectionMethod::Combined,
                Some(format!("{error:?}")),
                None,
            )
        }
    };

    let py_launcher_paths = where_command("py").unwrap_or_default();
    let path_probe = common_python_paths();
    let version_result = if !python_paths.is_empty() {
        run_command("python", &["--version"]).map(Some)
    } else if !py_launcher_paths.is_empty() {
        run_command("py", &["--version"]).map(Some)
    } else if let Some(path) = path_probe.as_ref() {
        run_path_command(path, &["--version"]).map(Some)
    } else {
        Ok(None)
    };

    let where_hit = !python_paths.is_empty() || !py_launcher_paths.is_empty();
    let path_hit = path_probe.is_some();
    let status = resolve_cli_status(where_hit, path_hit, version_result.clone());
    let output = version_result.ok().flatten();

    build_tool_status(
        "python",
        "Python",
        ToolCategory::Base,
        status,
        output.as_ref().and_then(extract_version_line),
        python_paths
            .first()
            .cloned()
            .or_else(|| py_launcher_paths.first().cloned())
            .or_else(|| path_probe.as_ref().map(|path| path.display().to_string())),
        DetectionMethod::Combined,
        output
            .as_ref()
            .filter(|item| item.exit_code != 0)
            .map(|item| item.stderr.trim().to_string())
            .filter(|message| !message.is_empty()),
        None,
    )
}

fn common_python_paths() -> Option<PathBuf> {
    let mut candidates = Vec::new();
    if let Some(user_profile) = user_profile_dir() {
        candidates.push(
            user_profile
                .join("AppData")
                .join("Local")
                .join("Programs")
                .join("Python")
                .join("Python311")
                .join("python.exe"),
        );
    }
    if let Some(program_files) = program_files_dir() {
        candidates.push(program_files.join("Python311").join("python.exe"));
    }
    first_existing_path(&candidates)
}
