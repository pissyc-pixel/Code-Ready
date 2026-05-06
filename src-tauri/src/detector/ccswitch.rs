use std::path::{Path, PathBuf};

use super::{
    shared::{appdata_dir, first_existing_path, localappdata_dir, now_iso_string, program_files_dir},
    DetectionMethod, ToolCategory, ToolInstallStatus, ToolStatus,
};

pub fn detect_with_config_path(configured_path: Option<PathBuf>) -> ToolStatus {
    let executable = resolve_detected_path(configured_path.clone());
    let configured_error = configured_path
        .as_ref()
        .and_then(|path| validate_configured_ccswitch_path(path).err());
    let status = if configured_error
        .as_deref()
        .is_some_and(|message| message.contains(".exe"))
    {
        ToolInstallStatus::Broken
    } else if executable.is_some() {
        ToolInstallStatus::Installed
    } else {
        ToolInstallStatus::Missing
    };

    ToolStatus {
        id: "ccswitch".to_string(),
        name: "ccSwitch".to_string(),
        category: ToolCategory::Ai,
        status,
        version: None,
        executable_path: executable.as_ref().map(|path| path.display().to_string()),
        detection_method: DetectionMethod::PathProbe,
        error_message: configured_error,
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

fn validate_configured_ccswitch_path(path: &Path) -> Result<(), String> {
    if !path.exists() {
        return Err("手动指定的 ccSwitch 路径不存在".to_string());
    }

    let is_exe = path
        .extension()
        .and_then(|item| item.to_str())
        .map(|item| item.eq_ignore_ascii_case("exe"))
        .unwrap_or(false);

    if !is_exe {
        return Err("手动指定的 ccSwitch 路径不是 .exe 文件".to_string());
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::{detect_with_config_path, resolve_detected_path, validate_configured_ccswitch_path};
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
            .contains("手动指定的 ccSwitch 路径不存在"));
    }

    #[test]
    fn unconfigured_missing_ccswitch_is_plain_missing_without_error() {
        let status = detect_with_config_path(None);

        assert!(matches!(status.status, super::ToolInstallStatus::Missing));
        assert!(status.error_message.is_none());
        assert!(status
            .suggestion
            .as_deref()
            .unwrap_or_default()
            .contains("never launches the GUI"));
    }

    #[test]
    fn configured_existing_exe_path_is_installed_via_path_probe() {
        let path = temp_test_path("ccswitch.exe");
        fs::write(&path, "placeholder").expect("write temp file");

        let status = detect_with_config_path(Some(path.clone()));

        assert!(matches!(status.status, super::ToolInstallStatus::Installed));
        assert_eq!(status.executable_path.as_deref(), Some(path.to_string_lossy().as_ref()));
        assert!(matches!(status.detection_method, super::DetectionMethod::PathProbe));

        let _ = fs::remove_file(path);
    }

    #[test]
    fn configured_non_exe_path_is_broken_with_clear_error() {
        let path = temp_test_path("ccswitch.txt");
        fs::write(&path, "placeholder").expect("write temp file");

        let status = detect_with_config_path(Some(path.clone()));

        assert!(matches!(status.status, super::ToolInstallStatus::Broken));
        assert!(status
            .error_message
            .as_deref()
            .unwrap_or_default()
            .contains(".exe"));

        let _ = fs::remove_file(path);
    }

    #[test]
    fn validate_configured_path_rejects_non_exe_without_launching() {
        let path = temp_test_path("ccswitch.txt");
        fs::write(&path, "placeholder").expect("write temp file");

        let error = validate_configured_ccswitch_path(&path).expect_err("non exe should fail");
        assert!(error.contains(".exe"));

        let _ = fs::remove_file(path);
    }
}
