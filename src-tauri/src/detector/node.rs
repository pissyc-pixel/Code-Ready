use std::path::PathBuf;

use super::{
    shared::{
        build_tool_status, extract_version_line, first_existing_path, program_files_dir,
        resolve_cli_status, run_command, run_path_command, where_command,
    },
    DetectionMethod, ToolCategory, ToolStatus,
};

pub fn detect_node() -> ToolStatus {
    let where_paths = match where_command("node") {
        Ok(paths) => paths,
        Err(error) => {
            return build_tool_status(
                "node",
                "Node.js",
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

    let path_probe = common_node_paths();
    let version_result = if let Some(path) = path_probe.as_ref().filter(|_| where_paths.is_empty()) {
        run_path_command(path, &["-v"]).map(Some)
    } else {
        run_command("node", &["-v"]).map(Some)
    };

    let status = resolve_cli_status(!where_paths.is_empty(), path_probe.is_some(), version_result.clone());
    let output = version_result.ok().flatten();

    build_tool_status(
        "node",
        "Node.js",
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
        None,
    )
}

fn common_node_paths() -> Option<PathBuf> {
    let mut candidates = Vec::new();
    if let Some(program_files) = program_files_dir() {
        candidates.push(program_files.join("nodejs").join("node.exe"));
    }
    first_existing_path(&candidates)
}
