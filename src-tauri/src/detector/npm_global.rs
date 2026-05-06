use std::path::PathBuf;

use super::shared::{appdata_dir, first_existing_path, run_command, user_profile_dir, ProbeError};

pub fn probe_npm_global_command(command: &str) -> Result<Option<PathBuf>, ProbeError> {
    let candidates = candidate_npm_global_paths(command);

    Ok(first_existing_path(&candidates))
}

pub fn candidate_npm_global_paths(command: &str) -> Vec<PathBuf> {
    candidate_npm_global_paths_from_sources(
        command,
        appdata_dir(),
        user_profile_dir(),
        npm_global_prefix_dir(),
    )
}

fn npm_global_prefix_dir() -> Option<PathBuf> {
    match run_command("npm", &["prefix", "-g"]) {
        Ok(output) if output.exit_code == 0 => {
            let prefix = output.stdout.trim();
            if prefix.is_empty() {
                None
            } else {
                Some(PathBuf::from(prefix))
            }
        }
        _ => None,
    }
}

fn candidate_npm_global_paths_from_sources(
    command: &str,
    appdata: Option<PathBuf>,
    user_profile: Option<PathBuf>,
    npm_prefix: Option<PathBuf>,
) -> Vec<PathBuf> {
    let mut candidates: Vec<PathBuf> = Vec::new();

    if let Some(appdata) = appdata {
        candidates.push(appdata.join("npm").join(format!("{command}.cmd")));
        candidates.push(appdata.join("npm").join(format!("{command}.ps1")));
        candidates.push(appdata.join("npm").join("node_modules").join(".bin").join(command));
        candidates.push(
            appdata
                .join("npm")
                .join("node_modules")
                .join(".bin")
                .join(format!("{command}.cmd")),
        );
    }

    if let Some(user_profile) = user_profile {
        candidates.push(
            user_profile
                .join(".npm-global")
                .join("bin")
                .join(format!("{command}.cmd")),
        );
    }

    if let Some(prefix) = npm_prefix {
        candidates.push(prefix.join(format!("{command}.cmd")));
        candidates.push(prefix.join(command));
    }

    candidates
}

#[cfg(test)]
mod tests {
    use super::candidate_npm_global_paths_from_sources;
    use std::path::PathBuf;

    #[test]
    fn windows_npm_global_candidates_include_cmd_wrappers_first() {
        let candidates = candidate_npm_global_paths_from_sources(
            "npm",
            Some(PathBuf::from("C:\\Users\\Tester\\AppData\\Roaming")),
            Some(PathBuf::from("C:\\Users\\Tester")),
            Some(PathBuf::from("D:\\npm-prefix")),
        );

        assert_eq!(
            candidates[0],
            PathBuf::from("C:\\Users\\Tester\\AppData\\Roaming\\npm\\npm.cmd")
        );
        assert!(candidates.contains(&PathBuf::from("D:\\npm-prefix\\npm.cmd")));
        assert!(candidates.contains(&PathBuf::from("D:\\npm-prefix\\npm")));
    }

    #[test]
    fn claude_codex_and_opencode_candidates_include_appdata_cmd_wrappers() {
        for command in ["claude", "codex", "opencode"] {
            let candidates = candidate_npm_global_paths_from_sources(
                command,
                Some(PathBuf::from(r"C:\Users\Tester\AppData\Roaming")),
                Some(PathBuf::from(r"C:\Users\Tester")),
                None,
            );

            assert_eq!(
                candidates[0],
                PathBuf::from(format!(
                    r"C:\Users\Tester\AppData\Roaming\npm\{command}.cmd"
                ))
            );
        }
    }

    #[test]
    fn npm_global_candidate_builder_does_not_require_prefix_probe_results() {
        let candidates = candidate_npm_global_paths_from_sources(
            "claude",
            Some(PathBuf::from(r"C:\Users\Tester\AppData\Roaming")),
            Some(PathBuf::from(r"C:\Users\Tester")),
            None,
        );

        assert!(candidates.contains(&PathBuf::from(
            r"C:\Users\Tester\AppData\Roaming\npm\claude.cmd"
        )));
    }
}
