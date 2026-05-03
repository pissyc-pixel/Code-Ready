use std::path::PathBuf;

use super::{
    shared::{appdata_dir, first_existing_path, localappdata_dir, now_iso_string, program_files_dir},
    DetectionMethod, ToolCategory, ToolInstallStatus, ToolStatus,
};

pub fn detect_with_config_path(configured_path: Option<PathBuf>) -> ToolStatus {
    let executable = resolve_detected_path(configured_path.clone());
    let has_invalid_configured_path = configured_path
        .as_ref()
        .map(|path| !path.exists())
        .unwrap_or(false);

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
        error_message: has_invalid_configured_path.then(|| {
            "Configured ccSwitch path does not exist; falling back to common install paths."
                .to_string()
        }),
        suggestion: Some(
            "ccSwitch detection only probes configured and common install paths. It never launches the GUI during detection."
                .to_string(),
        ),
        last_checked_at: now_iso_string(),
    }
}

pub fn resolve_detected_path(configured_path: Option<PathBuf>) -> Option<PathBuf> {
    let candidates = candidate_paths(configured_path);
    first_existing_path(&candidates)
}

fn candidate_paths(configured_path: Option<PathBuf>) -> Vec<PathBuf> {
    let mut candidates = Vec::new();

    if let Some(path) = configured_path {
        candidates.push(path);
    }

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

    candidates
}

#[cfg(test)]
mod tests {
    use super::{detect_with_config_path, resolve_detected_path};
    use std::{
        fs,
        path::PathBuf,
        time::{SystemTime, UNIX_EPOCH},
    };

    fn temp_test_path(file_name: &str) -> PathBuf {
        let unique = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system time")
            .as_nanos();
        std::env::temp_dir().join(format!("aicoding-{unique}-{file_name}"))
    }

    #[test]
    fn prefers_configured_ccswitch_path_when_it_exists() {
        let path = temp_test_path("ccswitch.exe");
        fs::write(&path, "placeholder").expect("write temp file");

        let detected = resolve_detected_path(Some(path.clone())).expect("configured path");
        assert_eq!(detected, path);

        let _ = fs::remove_file(path);
    }

    #[test]
    fn marks_missing_with_error_when_configured_path_does_not_exist() {
        let status = detect_with_config_path(Some(PathBuf::from("C:\\missing\\ccswitch.exe")));

        assert!(matches!(status.status, super::ToolInstallStatus::Missing));
        assert!(status
            .error_message
            .expect("configured path error")
            .contains("does not exist"));
    }
}
