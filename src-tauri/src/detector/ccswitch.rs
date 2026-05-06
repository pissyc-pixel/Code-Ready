use std::{
    fs::{self, OpenOptions},
    io::Write,
    path::{Path, PathBuf},
};

use super::{
    shared::{
        appdata_dir, extract_version_line, localappdata_dir, now_iso_string, program_files_dir,
        run_path_command, where_command,
    },
    DetectionMethod, ToolCategory, ToolInstallStatus, ToolStatus,
};

#[derive(Debug, Clone)]
struct CandidatePath {
    path: PathBuf,
    source: &'static str,
    detection_method: DetectionMethod,
}

pub fn detect_with_config_path(configured_path: Option<PathBuf>) -> ToolStatus {
    let mut trace = vec![format!(
        "[{}] starting ccSwitch detection",
        now_iso_string()
    )];
    let configured_error = configured_path
        .as_ref()
        .and_then(|path| configured_candidates(path).err());
    let selected = resolve_detected_candidate(configured_path, &mut trace);
    let version = selected
        .as_ref()
        .and_then(|candidate| detect_version(&candidate.path, &mut trace));

    if let Some(candidate) = selected.as_ref() {
        trace.push(format!(
            "selected [{}] {}",
            candidate.source,
            candidate.path.display()
        ));
    } else {
        trace.push("no ccSwitch executable candidate was found".to_string());
    }

    write_detection_log(&trace);

    ToolStatus {
        id: "ccswitch".to_string(),
        name: "ccSwitch".to_string(),
        category: ToolCategory::Ai,
        status: if selected.is_some() {
            ToolInstallStatus::Installed
        } else {
            ToolInstallStatus::Missing
        },
        version,
        executable_path: selected
            .as_ref()
            .map(|candidate| candidate.path.display().to_string()),
        detection_method: selected
            .as_ref()
            .map(|candidate| candidate.detection_method.clone())
            .unwrap_or(DetectionMethod::PathProbe),
        error_message: configured_error,
        suggestion: Some(
            "ccSwitch 检测会优先使用你保存的路径，并尝试 PATH 与常见安装位置。".to_string(),
        ),
        last_checked_at: now_iso_string(),
    }
}

pub fn resolve_detected_path(configured_path: Option<PathBuf>) -> Option<PathBuf> {
    resolve_detected_candidate(configured_path, &mut Vec::new()).map(|candidate| candidate.path)
}

fn resolve_detected_candidate(
    configured_path: Option<PathBuf>,
    trace: &mut Vec<String>,
) -> Option<CandidatePath> {
    let candidates = candidate_paths(configured_path, trace);
    first_existing_candidate(&candidates)
}

fn candidate_paths(configured_path: Option<PathBuf>, trace: &mut Vec<String>) -> Vec<CandidatePath> {
    let mut candidates = Vec::new();

    if let Some(path) = configured_path {
        match configured_candidates(&path) {
            Ok(configured_candidates) => {
                for candidate in configured_candidates {
                    push_unique_candidate(&mut candidates, candidate);
                }
            }
            Err(error) => {
                trace.push(format!("configured path rejected: {error}"));
            }
        }
    }

    for name in ["cc-switch", "cc-switch.exe", "ccswitch", "ccswitch.exe"] {
        match where_command(name) {
            Ok(paths) => {
                for path in paths.into_iter().map(PathBuf::from) {
                    push_unique_candidate(
                        &mut candidates,
                        CandidatePath {
                            path,
                            source: "PATH",
                            detection_method: DetectionMethod::Which,
                        },
                    );
                }
            }
            Err(error) => {
                trace.push(format!("where.exe {name} failed: {error}"));
            }
        }
    }

    for candidate in common_install_candidates() {
        push_unique_candidate(&mut candidates, candidate);
    }

    for candidate in &candidates {
        trace.push(format!(
            "candidate [{}] {} exists={}",
            candidate.source,
            candidate.path.display(),
            candidate.path.exists()
        ));
    }

    candidates
}

fn configured_candidates(path: &Path) -> Result<Vec<CandidatePath>, String> {
    if !path.exists() {
        return Err("手动指定的 ccSwitch 路径不存在".to_string());
    }

    if path.is_dir() {
        let mut candidates = Vec::new();
        for file_name in executable_file_names() {
            let candidate_path = path.join(file_name);
            if candidate_path.is_file() {
                push_unique_candidate(
                    &mut candidates,
                    CandidatePath {
                        path: candidate_path,
                        source: "configured-directory",
                        detection_method: DetectionMethod::PathProbe,
                    },
                );
            }
        }

        if candidates.is_empty() {
            return Err("手动指定的 ccSwitch 目录中未找到可执行文件".to_string());
        }

        return Ok(candidates);
    }

    if is_executable_file(path) {
        return Ok(vec![CandidatePath {
            path: path.to_path_buf(),
            source: "configured-file",
            detection_method: DetectionMethod::PathProbe,
        }]);
    }

    Err("手动指定的 ccSwitch 路径不是 .exe 文件".to_string())
}

fn common_install_candidates() -> Vec<CandidatePath> {
    let mut candidates = Vec::new();

    if let Some(appdata) = appdata_dir() {
        push_variant_paths(
            &mut candidates,
            appdata.join("ai-coding-installer").join("ccswitch"),
            "managed-appdata",
        );
        push_variant_paths(&mut candidates, appdata.join("ccswitch"), "appdata");
    }

    if let Some(localappdata) = localappdata_dir() {
        push_variant_paths(&mut candidates, localappdata.join("ccswitch"), "localappdata");
        push_variant_paths(&mut candidates, localappdata.join("cc-switch"), "localappdata");
    }

    if let Some(program_files) = program_files_dir() {
        push_variant_paths(&mut candidates, program_files.join("ccswitch"), "program-files");
        push_variant_paths(&mut candidates, program_files.join("cc-switch"), "program-files");
    }

    if let Some(program_files_x86) = program_files_x86_dir() {
        push_variant_paths(
            &mut candidates,
            program_files_x86.join("ccswitch"),
            "program-files-x86",
        );
        push_variant_paths(
            &mut candidates,
            program_files_x86.join("cc-switch"),
            "program-files-x86",
        );
    }

    push_variant_paths(&mut candidates, PathBuf::from(r"D:\ccSwitch"), "custom-d-drive");
    push_variant_paths(&mut candidates, PathBuf::from(r"C:\ccSwitch"), "custom-c-drive");

    if let Some(current_exe_dir) = current_exe_dir() {
        push_variant_paths(&mut candidates, current_exe_dir.clone(), "app-directory");
        push_variant_paths(
            &mut candidates,
            current_exe_dir.join("ccswitch"),
            "app-directory",
        );
    }

    candidates
}

fn push_variant_paths(candidates: &mut Vec<CandidatePath>, directory: PathBuf, source: &'static str) {
    for file_name in executable_file_names() {
        push_unique_candidate(
            candidates,
            CandidatePath {
                path: directory.join(file_name),
                source,
                detection_method: DetectionMethod::PathProbe,
            },
        );
    }
}

fn executable_file_names() -> [&'static str; 2] {
    ["cc-switch.exe", "ccswitch.exe"]
}

fn detect_version(path: &Path, trace: &mut Vec<String>) -> Option<String> {
    for args in [["--version"], ["-V"], ["version"]] {
        match run_path_command(path, &args) {
            Ok(output) => {
                trace.push(format!(
                    "version probe {:?} exit={} stdout='{}' stderr='{}'",
                    args,
                    output.exit_code,
                    output.stdout.trim(),
                    output.stderr.trim()
                ));
                if output.exit_code == 0 {
                    if let Some(version) = extract_version_line(&output) {
                        return Some(version);
                    }
                }
            }
            Err(error) => {
                trace.push(format!("version probe {:?} failed: {error}", args));
            }
        }
    }

    None
}

fn first_existing_candidate(candidates: &[CandidatePath]) -> Option<CandidatePath> {
    candidates
        .iter()
        .find(|candidate| candidate.path.is_file())
        .cloned()
}

fn push_unique_candidate(candidates: &mut Vec<CandidatePath>, candidate: CandidatePath) {
    if !candidates.iter().any(|existing| existing.path == candidate.path) {
        candidates.push(candidate);
    }
}

fn is_executable_file(path: &Path) -> bool {
    path.extension()
        .and_then(|item| item.to_str())
        .map(|item| item.eq_ignore_ascii_case("exe"))
        .unwrap_or(false)
}

fn current_exe_dir() -> Option<PathBuf> {
    std::env::current_exe()
        .ok()
        .and_then(|path| path.parent().map(Path::to_path_buf))
}

fn program_files_x86_dir() -> Option<PathBuf> {
    std::env::var_os("ProgramFiles(x86)").map(PathBuf::from)
}

fn write_detection_log(lines: &[String]) {
    let Some(log_path) = detection_log_path() else {
        return;
    };

    if let Some(parent) = log_path.parent() {
        let _ = fs::create_dir_all(parent);
    }

    if let Ok(mut file) = OpenOptions::new().create(true).append(true).open(log_path) {
        for line in lines {
            let _ = writeln!(file, "{line}");
        }
    }
}

fn detection_log_path() -> Option<PathBuf> {
    appdata_dir().map(|appdata| {
        appdata
            .join("ai-coding-installer")
            .join("logs")
            .join("ccswitch-detect.log")
    })
}

#[cfg(test)]
mod tests {
    use super::{
        configured_candidates, detect_with_config_path, detect_version, resolve_detected_path,
    };
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
    fn accepts_configured_directory_and_resolves_known_executable_names() {
        let dir = temp_test_path("ccswitch-dir");
        fs::create_dir_all(&dir).expect("create temp dir");
        let executable = dir.join("cc-switch.exe");
        fs::write(&executable, "placeholder").expect("write temp exe");

        let detected = resolve_detected_path(Some(dir.clone())).expect("configured directory");
        assert_eq!(detected, executable);

        let _ = fs::remove_dir_all(dir);
    }

    #[test]
    fn marks_missing_with_error_when_configured_path_does_not_exist() {
        let status = detect_with_config_path(Some(PathBuf::from("C:\\missing\\ccswitch.exe")));

        assert!(!matches!(status.status, super::ToolInstallStatus::DetectFailed));
        assert!(status
            .error_message
            .expect("configured path error")
            .contains("路径不存在"));
    }

    #[test]
    fn unconfigured_missing_ccswitch_is_plain_missing_without_error() {
        let status = detect_with_config_path(None);

        assert!(status.error_message.is_none());
        assert_eq!(status.id, "ccswitch");
        assert_eq!(status.name, "ccSwitch");
    }

    #[test]
    fn configured_existing_exe_path_is_installed_even_without_version() {
        let path = temp_test_path("ccswitch.exe");
        fs::write(&path, "placeholder").expect("write temp file");

        let status = detect_with_config_path(Some(path.clone()));

        assert!(matches!(status.status, super::ToolInstallStatus::Installed));
        assert_eq!(status.version, None);
        assert_eq!(status.executable_path.as_deref(), Some(path.to_string_lossy().as_ref()));

        let _ = fs::remove_file(path);
    }

    #[test]
    fn configured_non_exe_path_is_missing_with_clear_error() {
        let path = temp_test_path("ccswitch.txt");
        fs::write(&path, "placeholder").expect("write temp file");

        let status = detect_with_config_path(Some(path.clone()));

        assert!(!matches!(status.status, super::ToolInstallStatus::DetectFailed));
        assert!(status
            .error_message
            .as_deref()
            .unwrap_or_default()
            .contains(".exe"));

        let _ = fs::remove_file(path);
    }

    #[test]
    fn configured_directory_without_executable_returns_error() {
        let dir = temp_test_path("ccswitch-empty-dir");
        fs::create_dir_all(&dir).expect("create temp dir");

        let error = configured_candidates(&dir).expect_err("directory without exe should fail");
        assert!(error.contains("目录中未找到"));

        let _ = fs::remove_dir_all(dir);
    }

    #[test]
    fn version_probe_tries_fallback_arguments() {
        let path = temp_test_path("ccswitch.cmd");
        fs::write(
            &path,
            "@echo off\r\nif \"%1\"==\"version\" (\r\n  echo ccSwitch 0.4.1\r\n  exit /b 0\r\n)\r\nexit /b 1\r\n",
        )
        .expect("write temp cmd");

        let version = detect_version(&path, &mut Vec::new());
        assert_eq!(version.as_deref(), Some("ccSwitch 0.4.1"));

        let _ = fs::remove_file(path);
    }
}
