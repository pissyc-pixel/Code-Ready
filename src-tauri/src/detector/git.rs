use std::path::PathBuf;

use super::{
    shared::{
        build_tool_status, extract_version_line, first_existing_path, program_files_dir,
        resolve_cli_status, run_command, run_path_command, where_command,
    },
    DetectionMethod, ToolCategory, ToolStatus,
};

pub fn detect() -> ToolStatus {
    let where_paths = match where_command("git") {
        Ok(paths) => paths,
        Err(error) => {
            return build_tool_status(
                "git",
                "Git / Git Bash",
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

    let path_probe = common_git_paths();
    let version_result = if let Some(path) = path_probe.as_ref().filter(|_| where_paths.is_empty()) {
        run_path_command(path, &["--version"]).map(Some)
    } else {
        run_command("git", &["--version"]).map(Some)
    };

    let status = resolve_cli_status(!where_paths.is_empty(), path_probe.is_some(), version_result.clone());
    let output = version_result.ok().flatten();
    let suggestion = git_bash_suggestion();

    build_tool_status(
        "git",
        "Git / Git Bash",
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

fn common_git_paths() -> Option<PathBuf> {
    let mut candidates = Vec::new();
    if let Some(program_files) = program_files_dir() {
        candidates.push(program_files.join("Git").join("cmd").join("git.exe"));
        candidates.push(program_files.join("Git").join("bin").join("git.exe"));
    }
    first_existing_path(&candidates)
}

fn git_bash_suggestion() -> Option<String> {
    let mut candidates = Vec::new();
    if let Some(program_files) = program_files_dir() {
        candidates.push(program_files.join("Git").join("git-bash.exe"));
        candidates.push(program_files.join("Git").join("bin").join("bash.exe"));
    }
    let git_bash = first_existing_path(&candidates);
    git_bash.map(|path| format!("Git Bash 已发现：{}", path.display()))
}
