use std::path::PathBuf;

use super::shared::{appdata_dir, first_existing_path, run_command, user_profile_dir, ProbeError};

pub fn probe_npm_global_command(command: &str) -> Result<Option<PathBuf>, ProbeError> {
    let mut candidates: Vec<PathBuf> = Vec::new();

    if let Some(appdata) = appdata_dir() {
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

    if let Some(user_profile) = user_profile_dir() {
        candidates.push(
            user_profile
                .join(".npm-global")
                .join("bin")
                .join(format!("{command}.cmd")),
        );
    }

    if let Ok(output) = run_command("npm", &["prefix", "-g"]) {
        if output.exit_code == 0 {
            let prefix = output.stdout.trim();
            if !prefix.is_empty() {
                candidates.push(PathBuf::from(prefix).join(format!("{command}.cmd")));
                candidates.push(PathBuf::from(prefix).join(command));
            }
        }
    }

    Ok(first_existing_path(&candidates))
}
