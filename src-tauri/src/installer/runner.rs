use std::{
    fs::{self, File},
    io::{BufRead, BufReader, Write},
    path::PathBuf,
    process::{Command, Stdio},
    sync::{Arc, Mutex},
    thread,
    time::{Duration, Instant},
};

use tauri::{AppHandle, Emitter, Manager};

use crate::{
    detector::ToolInstallStatus,
    installer::{
        now_timestamp, InstallPhase, InstallProgressEvent, InstallResult, InstallStatusEvent,
    },
    logger::redact::redact_line,
    process::kill_tree::{kill_process_tree, KillTreeOutcome},
    state::AppState,
};

#[derive(Debug, Clone)]
pub struct InstallCommandSpec {
    pub program: String,
    pub args: Vec<String>,
    pub timeout: Duration,
}

pub fn build_command_spec(tool_id: &str) -> Result<InstallCommandSpec, String> {
    match tool_id {
        "__test_success__" => Ok(InstallCommandSpec {
            program: "powershell".to_string(),
            args: vec![
                "-NoProfile".to_string(),
                "-Command".to_string(),
                "Write-Output 'token=abc123'; Start-Sleep -Milliseconds 100; Write-Output 'Authorization: Bearer secret-token'; Start-Sleep -Milliseconds 100".to_string(),
            ],
            timeout: Duration::from_secs(10),
        }),
        "__test_timeout__" => Ok(InstallCommandSpec {
            program: "powershell".to_string(),
            args: vec![
                "-NoProfile".to_string(),
                "-Command".to_string(),
                "Start-Sleep -Seconds 3".to_string(),
            ],
            timeout: Duration::from_millis(500),
        }),
        "__test_cancel__" => Ok(InstallCommandSpec {
            program: "powershell".to_string(),
            args: vec![
                "-NoProfile".to_string(),
                "-Command".to_string(),
                "1..20 | ForEach-Object { Write-Output \"line=$_\"; Start-Sleep -Milliseconds 200 }".to_string(),
            ],
            timeout: Duration::from_secs(10),
        }),
        _ => Err(format!(
            "installer for {tool_id} is not available in V0.3 commit 3; real installers land in commit 4"
        )),
    }
}

pub fn spawn_install_runner(app: AppHandle, tool_id: String, spec: InstallCommandSpec) {
    thread::spawn(move || {
        if let Err(error) = run_install_task(app.clone(), tool_id.clone(), spec) {
            let _ = app.state::<AppState>().clear_install(&tool_id);
            let _ = app.emit(
                "install:status",
                InstallStatusEvent {
                    tool_id,
                    status: ToolInstallStatus::InstallFailed,
                    phase: InstallPhase::Failed,
                    error_message: Some(error),
                    suggestion: None,
                    result: None,
                    timestamp: now_timestamp(),
                },
            );
        }
    });
}

fn run_install_task(app: AppHandle, tool_id: String, spec: InstallCommandSpec) -> Result<(), String> {
    let state = app.state::<AppState>();
    let logger = TaskLogger::new(&tool_id)?;
    let started_at = Instant::now();

    let mut child = Command::new(&spec.program)
        .args(&spec.args)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|error| error.to_string())?;

    let pid = child.id();
    state.update_install_pid(&tool_id, pid)?;
    emit_status(
        &app,
        InstallStatusEvent {
            tool_id: tool_id.clone(),
            status: ToolInstallStatus::Installing,
            phase: InstallPhase::Started,
            error_message: None,
            suggestion: None,
            result: None,
            timestamp: now_timestamp(),
        },
    )?;

    let stdout_handle = child.stdout.take().map(|stdout| {
        spawn_stream_reader(
            app.clone(),
            logger.clone(),
            tool_id.clone(),
            "stdout",
            stdout,
        )
    });
    let stderr_handle = child.stderr.take().map(|stderr| {
        spawn_stream_reader(
            app.clone(),
            logger.clone(),
            tool_id.clone(),
            "stderr",
            stderr,
        )
    });

    if state.is_cancel_requested(&tool_id)? {
        let _ = kill_process_tree(pid);
    }

    let mut cancel_effective = false;
    let mut timed_out = false;

    let exit_code = loop {
        if started_at.elapsed() >= spec.timeout {
            timed_out = true;
            let _ = kill_process_tree(pid);
        }

        if state.is_cancel_requested(&tool_id)? {
            match kill_process_tree(pid) {
                Ok(KillTreeOutcome::Killed) => {
                    cancel_effective = true;
                }
                Ok(KillTreeOutcome::AlreadyExited) => {}
                Err(error) => return Err(error),
            }
        }

        match child.try_wait().map_err(|error| error.to_string())? {
            Some(status) => break status.code(),
            None => thread::sleep(Duration::from_millis(100)),
        }
    };

    if let Some(handle) = stdout_handle {
        let _ = handle.join();
    }
    if let Some(handle) = stderr_handle {
        let _ = handle.join();
    }

    let duration_ms = started_at.elapsed().as_millis() as u64;
    let final_event = if timed_out {
        InstallStatusEvent {
            tool_id: tool_id.clone(),
            status: ToolInstallStatus::InstallFailed,
            phase: InstallPhase::Timeout,
            error_message: Some("install task timed out".to_string()),
            suggestion: None,
            result: Some(InstallResult {
                exit_code,
                duration_ms: Some(duration_ms),
            }),
            timestamp: now_timestamp(),
        }
    } else if cancel_effective {
        InstallStatusEvent {
            tool_id: tool_id.clone(),
            status: ToolInstallStatus::Missing,
            phase: InstallPhase::Cancelled,
            error_message: None,
            suggestion: Some("瀹夎宸插彇娑堬紝鍙互閲嶆柊鍚姩銆?".to_string()),
            result: Some(InstallResult {
                exit_code,
                duration_ms: Some(duration_ms),
            }),
            timestamp: now_timestamp(),
        }
    } else if exit_code == Some(0) {
        InstallStatusEvent {
            tool_id: tool_id.clone(),
            status: ToolInstallStatus::Installed,
            phase: InstallPhase::Success,
            error_message: None,
            suggestion: Some(
                "V0.3 commit 3 宸插畬鎴愬悗鍙板畨瑁呴摼璺獙璇侊紝鐪熷疄瀹夎鍣ㄥ皢鍦ㄤ笅涓€涓彁浜や腑鎺ュ叆銆?"
                    .to_string(),
            ),
            result: Some(InstallResult {
                exit_code,
                duration_ms: Some(duration_ms),
            }),
            timestamp: now_timestamp(),
        }
    } else {
        InstallStatusEvent {
            tool_id: tool_id.clone(),
            status: ToolInstallStatus::InstallFailed,
            phase: InstallPhase::Failed,
            error_message: Some("install task exited with a non-zero code".to_string()),
            suggestion: None,
            result: Some(InstallResult {
                exit_code,
                duration_ms: Some(duration_ms),
            }),
            timestamp: now_timestamp(),
        }
    };

    state.clear_install(&tool_id)?;
    emit_status(&app, final_event)?;
    Ok(())
}

fn emit_status(app: &AppHandle, event: InstallStatusEvent) -> Result<(), String> {
    app.emit("install:status", event)
        .map_err(|error| error.to_string())
}

fn spawn_stream_reader<R>(
    app: AppHandle,
    logger: TaskLogger,
    tool_id: String,
    stream: &'static str,
    reader: R,
) -> thread::JoinHandle<()>
where
    R: std::io::Read + Send + 'static,
{
    thread::spawn(move || {
        let buffered = BufReader::new(reader);
        for line in buffered.lines().map_while(Result::ok) {
            let redacted = redact_line(&line);
            let _ = app.emit(
                "install:progress",
                InstallProgressEvent {
                    tool_id: tool_id.clone(),
                    phase: InstallPhase::Running,
                    line: redacted.clone(),
                    stream: stream.to_string(),
                    timestamp: now_timestamp(),
                },
            );
            let _ = logger.write_line(stream, &redacted);
        }
    })
}

#[derive(Clone)]
struct TaskLogger {
    file: Arc<Mutex<File>>,
}

impl TaskLogger {
    fn new(tool_id: &str) -> Result<Self, String> {
        let path = log_path(tool_id)?;
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).map_err(|error| error.to_string())?;
        }
        let file = File::create(path).map_err(|error| error.to_string())?;
        Ok(Self {
            file: Arc::new(Mutex::new(file)),
        })
    }

    fn write_line(&self, stream: &str, line: &str) -> Result<(), String> {
        let mut file = self
            .file
            .lock()
            .map_err(|_| "install log poisoned".to_string())?;
        writeln!(file, "[{stream}] {line}").map_err(|error| error.to_string())
    }
}

fn log_path(tool_id: &str) -> Result<PathBuf, String> {
    let appdata = std::env::var_os("APPDATA")
        .map(PathBuf::from)
        .ok_or_else(|| "APPDATA is not available".to_string())?;
    Ok(appdata
        .join("ai-coding-installer")
        .join("logs")
        .join(format!(
            "{tool_id}-{}.log",
            chrono::Local::now().format("%Y%m%d-%H%M%S")
        )))
}

#[cfg(test)]
mod tests {
    use super::build_command_spec;

    #[test]
    fn supports_test_runner_specs() {
        let spec = build_command_spec("__test_success__").expect("support test command");
        assert_eq!(spec.program, "powershell");
        assert!(!spec.args.is_empty());
    }
}
