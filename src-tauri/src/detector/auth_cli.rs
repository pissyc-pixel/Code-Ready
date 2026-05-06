use std::path::PathBuf;

use super::{
    npm_global,
    shared::{
        appdata_dir, build_tool_status, command_error_message, extract_version_line,
        resolve_auth_sensitive_status, run_path_command, where_command,
    },
    DetectionMethod, ToolCategory, ToolInstallStatus, ToolStatus,
};

#[derive(Debug, Clone)]
struct CliCandidate {
    path: PathBuf,
    detection_method: DetectionMethod,
}

pub(crate) fn detect_auth_sensitive_cli(command: &str, name: &str) -> ToolStatus {
    let where_cmd_paths = match where_command(&format!("{command}.cmd")) {
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

    let where_bare_paths = match where_command(command) {
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

    let candidates =
        prioritized_ai_cli_candidates(command, &where_cmd_paths, &where_bare_paths, npm_global::candidate_npm_global_paths(command));
    let selected = first_existing_candidate(&candidates);
    let version_result = if let Some(candidate) = selected.as_ref() {
        run_path_command(&candidate.path, &["--version"]).map(Some)
    } else {
        Ok(None)
    };
    let where_hit = !where_cmd_paths.is_empty() || !where_bare_paths.is_empty();
    let status = resolve_auth_sensitive_status(where_hit, selected.is_some(), version_result.clone());
    let output = version_result.clone().ok().flatten();
    let needs_config = matches!(
        status,
        ToolInstallStatus::Installed | ToolInstallStatus::InstalledButPathMissing
    );
    let show_error = matches!(status, ToolInstallStatus::Broken | ToolInstallStatus::DetectFailed);

    build_tool_status(
        command,
        name,
        ToolCategory::Ai,
        status,
        output.as_ref().and_then(extract_version_line),
        selected
            .as_ref()
            .map(|candidate| candidate.path.display().to_string()),
        selected
            .as_ref()
            .map(|candidate| candidate.detection_method.clone())
            .unwrap_or(DetectionMethod::NpmGlobalProbe),
        output
            .as_ref()
            .filter(|item| item.exit_code != 0)
            .and_then(command_error_message)
            .filter(|_| show_error),
        needs_config.then(|| {
            "Detected CLI files, but login or local configuration may still be required."
                .to_string()
        }),
    )
}

fn prioritized_ai_cli_candidates(
    command: &str,
    where_cmd_paths: &[String],
    where_bare_paths: &[String],
    npm_global_candidates: Vec<PathBuf>,
) -> Vec<CliCandidate> {
    let mut candidates = Vec::new();

    if let Some(appdata_cmd) = appdata_cli_cmd_path(command) {
        push_unique_candidate(
            &mut candidates,
            CliCandidate {
                path: appdata_cmd,
                detection_method: DetectionMethod::NpmGlobalProbe,
            },
        );
    }

    for path in where_cmd_paths.iter().map(PathBuf::from) {
        push_unique_candidate(
            &mut candidates,
            CliCandidate {
                path,
                detection_method: DetectionMethod::Which,
            },
        );
    }

    for path in where_bare_paths.iter().map(PathBuf::from) {
        push_unique_candidate(
            &mut candidates,
            CliCandidate {
                path,
                detection_method: DetectionMethod::Which,
            },
        );
    }

    for path in npm_global_candidates {
        push_unique_candidate(
            &mut candidates,
            CliCandidate {
                path,
                detection_method: DetectionMethod::NpmGlobalProbe,
            },
        );
    }

    candidates
}

fn appdata_cli_cmd_path(command: &str) -> Option<PathBuf> {
    appdata_dir().map(|appdata| appdata.join("npm").join(format!("{command}.cmd")))
}

fn push_unique_candidate(candidates: &mut Vec<CliCandidate>, candidate: CliCandidate) {
    if !candidates.iter().any(|existing| existing.path == candidate.path) {
        candidates.push(candidate);
    }
}

fn first_existing_candidate(candidates: &[CliCandidate]) -> Option<CliCandidate> {
    candidates
        .iter()
        .find(|candidate| candidate.path.exists())
        .cloned()
}

#[cfg(test)]
mod tests {
    use super::{first_existing_candidate, prioritized_ai_cli_candidates};
    use crate::detector::{
        shared::{resolve_auth_sensitive_status, CommandOutput},
        DetectionMethod, ToolInstallStatus,
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
        std::env::temp_dir().join(format!("aicoding-auth-cli-{unique}-{file_name}"))
    }

    #[test]
    fn candidate_order_prefers_appdata_cmd_for_all_ai_clis() {
        let appdata_dir = std::env::var("APPDATA").expect("APPDATA");

        for command in ["claude", "codex", "opencode"] {
            let candidates = prioritized_ai_cli_candidates(
                command,
                &[format!(r"C:\FromWhere\{command}.cmd")],
                &[format!(r"C:\FromWhere\{command}")],
                vec![PathBuf::from(format!(r"D:\Extra\{command}.cmd"))],
            );

            assert_eq!(
                candidates[0].path,
                PathBuf::from(appdata_dir.clone())
                    .join("npm")
                    .join(format!("{command}.cmd"))
            );
            assert!(matches!(
                candidates[0].detection_method,
                DetectionMethod::NpmGlobalProbe
            ));
        }
    }

    #[test]
    fn path_probe_success_without_where_hit_maps_to_installed_but_path_missing() {
        let status = resolve_auth_sensitive_status(
            false,
            true,
            Ok(Some(CommandOutput {
                exit_code: 0,
                stdout: "1.2.3".to_string(),
                stderr: String::new(),
            })),
        );

        assert!(matches!(
            status,
            ToolInstallStatus::InstalledButPathMissing
        ));
    }

    #[test]
    fn auth_output_from_path_probe_does_not_become_broken() {
        let status = resolve_auth_sensitive_status(
            false,
            true,
            Ok(Some(CommandOutput {
                exit_code: 1,
                stdout: String::new(),
                stderr: "please sign in or provide an API key".to_string(),
            })),
        );

        assert!(matches!(
            status,
            ToolInstallStatus::InstalledButPathMissing
        ));
    }

    #[test]
    fn first_existing_candidate_prefers_existing_cmd_wrapper() {
        let path = temp_test_path("claude.cmd");
        fs::write(&path, "@echo off").expect("write temp cmd");

        let candidates = vec![
            super::CliCandidate {
                path: path.clone(),
                detection_method: DetectionMethod::NpmGlobalProbe,
            },
            super::CliCandidate {
                path: PathBuf::from(r"C:\missing\claude"),
                detection_method: DetectionMethod::Which,
            },
        ];

        let selected = first_existing_candidate(&candidates).expect("existing candidate");
        assert_eq!(selected.path, path);

        let _ = fs::remove_file(selected.path);
    }
}
