use std::path::PathBuf;

use super::{
    shared::{appdata_dir, first_existing_path, localappdata_dir, program_files_dir},
    DetectionMethod, ToolCategory, ToolInstallStatus, ToolStatus,
};

pub fn detect() -> ToolStatus {
    let executable = common_paths();

    ToolStatus {
        id: "ccswitch".to_string(),
        name: "ccSwitch".to_string(),
        category: ToolCategory::Ai,
        status: if executable.is_some() {
            ToolInstallStatus::Installed
        } else {
            ToolInstallStatus::Missing
        },
        version: None,
        executable_path: executable.as_ref().map(|path| path.display().to_string()),
        detection_method: DetectionMethod::PathProbe,
        error_message: None,
        suggestion: Some(
            "V0.2 only probes common install paths. It does not launch the ccSwitch GUI."
                .to_string(),
        ),
        last_checked_at: super::shared::now_iso_string(),
    }
}

fn common_paths() -> Option<PathBuf> {
    let mut candidates = Vec::new();

    if let Some(appdata) = appdata_dir() {
        candidates.push(
            appdata
                .join("ai-coding-installer")
                .join("ccswitch")
                .join("ccswitch.exe"),
        );
    }

    if let Some(localappdata) = localappdata_dir() {
        candidates.push(localappdata.join("ccswitch").join("ccswitch.exe"));
    }

    if let Some(program_files) = program_files_dir() {
        candidates.push(program_files.join("ccswitch").join("ccswitch.exe"));
    }

    first_existing_path(&candidates)
}
