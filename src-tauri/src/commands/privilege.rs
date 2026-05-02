use std::process::Command;

fn admin_probe_command() -> Vec<String> {
    vec![
        "-NoProfile".to_string(),
        "-Command".to_string(),
        "[bool](([Security.Principal.WindowsPrincipal] [Security.Principal.WindowsIdentity]::GetCurrent()).IsInRole([Security.Principal.WindowsBuiltInRole]::Administrator))".to_string(),
    ]
}

#[tauri::command]
pub async fn is_admin() -> Result<bool, String> {
    let output = Command::new("powershell")
        .args(admin_probe_command())
        .output()
        .map_err(|error| error.to_string())?;

    if !output.status.success() {
        return Err(String::from_utf8_lossy(&output.stderr).trim().to_string());
    }

    Ok(String::from_utf8_lossy(&output.stdout).trim().eq_ignore_ascii_case("true"))
}

#[tauri::command]
pub async fn restart_as_admin() -> Result<(), String> {
    let exe = std::env::current_exe().map_err(|error| error.to_string())?;
    let command = format!(
        "Start-Process -Verb RunAs -FilePath '{}'",
        exe.display().to_string().replace('\'', "''")
    );

    let output = Command::new("powershell")
        .args(["-NoProfile", "-Command", &command])
        .output()
        .map_err(|error| error.to_string())?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
        return Err(if stderr.is_empty() {
            "failed to request administrator restart".to_string()
        } else {
            stderr
        });
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn builds_admin_probe_command() {
        let args = admin_probe_command();
        assert_eq!(args[0], "-NoProfile");
        assert_eq!(args[1], "-Command");
        assert!(args[2].contains("Administrator"));
    }
}
