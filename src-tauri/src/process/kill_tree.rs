use std::process::Command;

pub fn build_taskkill_args(pid: u32) -> Vec<String> {
    vec![
        "/F".to_string(),
        "/T".to_string(),
        "/PID".to_string(),
        pid.to_string(),
    ]
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum KillTreeOutcome {
    Killed,
    AlreadyExited,
}

pub fn kill_process_tree(pid: u32) -> Result<KillTreeOutcome, String> {
    let output = Command::new("taskkill")
        .args(build_taskkill_args(pid))
        .output()
        .map_err(|error| error.to_string())?;

    if output.status.success() {
        return Ok(KillTreeOutcome::Killed);
    }

    let stderr = String::from_utf8_lossy(&output.stderr);
    let stdout = String::from_utf8_lossy(&output.stdout);
    let message = format!("{stdout}\n{stderr}").to_lowercase();
    if message.contains("not found") || message.contains("no running instance") {
        return Ok(KillTreeOutcome::AlreadyExited);
    }

    Err(message.trim().to_string())
}

#[cfg(test)]
mod tests {
    use super::build_taskkill_args;

    #[test]
    fn builds_windows_process_tree_kill_args() {
        assert_eq!(
            build_taskkill_args(4321),
            vec![
                "/F".to_string(),
                "/T".to_string(),
                "/PID".to_string(),
                "4321".to_string()
            ]
        );
    }
}
