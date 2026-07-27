use std::ffi::OsString;
use std::io::Read;
use std::path::PathBuf;
use std::process::{Command, Stdio};
use std::time::{Duration, Instant};

pub const PROBE_TIMEOUT: Duration = Duration::from_secs(3);
pub const OUTPUT_LIMIT_BYTES: usize = 64 * 1024;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ProcessRequest {
    pub executable: PathBuf,
    pub args: Vec<OsString>,
    pub timeout: Duration,
    pub environment: Vec<(OsString, OsString)>,
}

impl ProcessRequest {
    pub fn version_probe(path: impl Into<PathBuf>) -> Self {
        Self {
            executable: path.into(),
            args: vec![OsString::from("--version")],
            timeout: PROBE_TIMEOUT,
            environment: vec![],
        }
    }

    pub fn apple_developer_dir_probe() -> Self {
        Self {
            executable: PathBuf::from("/usr/bin/xcode-select"),
            args: vec![OsString::from("--print-path")],
            timeout: PROBE_TIMEOUT,
            environment: vec![],
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum LaunchFailureKind {
    NotFound,
    PermissionDenied,
    Other,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ProcessOutcome {
    Exited {
        code: Option<i32>,
        stdout: Vec<u8>,
        stderr: Vec<u8>,
        stdout_truncated: bool,
        stderr_truncated: bool,
    },
    TimedOut,
    LaunchFailed(LaunchFailureKind),
}

pub trait ProcessRunner: Send + Sync {
    fn run(&self, request: ProcessRequest) -> ProcessOutcome;
}

#[derive(Clone, Copy, Debug, Default)]
pub struct NativeProcessRunner;

impl ProcessRunner for NativeProcessRunner {
    fn run(&self, request: ProcessRequest) -> ProcessOutcome {
        let mut command = Command::new(&request.executable);
        command
            .args(&request.args)
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());
        for (key, value) in &request.environment {
            command.env(key, value);
        }

        let mut child = match command.spawn() {
            Ok(child) => child,
            Err(error) => return ProcessOutcome::LaunchFailed(map_launch_failure(&error)),
        };

        let stdout = child.stdout.take().expect("stdout was configured as piped");
        let stderr = child.stderr.take().expect("stderr was configured as piped");
        let stdout_reader = std::thread::spawn(move || read_bounded(stdout));
        let stderr_reader = std::thread::spawn(move || read_bounded(stderr));
        let deadline = Instant::now() + request.timeout;

        let exit_status = loop {
            match child.try_wait() {
                Ok(Some(status)) => break status,
                Ok(None) if Instant::now() >= deadline => {
                    let _ = child.kill();
                    let _ = child.wait();
                    join_reader(stdout_reader);
                    join_reader(stderr_reader);
                    return ProcessOutcome::TimedOut;
                }
                Ok(None) => {
                    let remaining = deadline.saturating_duration_since(Instant::now());
                    std::thread::sleep(if remaining > Duration::from_millis(10) {
                        Duration::from_millis(10)
                    } else {
                        remaining
                    });
                }
                Err(_) => {
                    let _ = child.kill();
                    let _ = child.wait();
                    join_reader(stdout_reader);
                    join_reader(stderr_reader);
                    return ProcessOutcome::LaunchFailed(LaunchFailureKind::Other);
                }
            }
        };

        let stdout = join_reader(stdout_reader);
        let stderr = join_reader(stderr_reader);
        ProcessOutcome::Exited {
            code: exit_status.code(),
            stdout: stdout.bytes,
            stderr: stderr.bytes,
            stdout_truncated: stdout.truncated,
            stderr_truncated: stderr.truncated,
        }
    }
}

#[derive(Debug, Default)]
struct BoundedOutput {
    bytes: Vec<u8>,
    truncated: bool,
}

fn read_bounded<R: Read>(mut reader: R) -> BoundedOutput {
    let mut output = BoundedOutput {
        bytes: Vec::with_capacity(OUTPUT_LIMIT_BYTES),
        truncated: false,
    };
    let mut buffer = [0_u8; 8 * 1024];

    loop {
        match reader.read(&mut buffer) {
            Ok(0) => break,
            Ok(bytes_read) => {
                let remaining = OUTPUT_LIMIT_BYTES.saturating_sub(output.bytes.len());
                let retained = bytes_read.min(remaining);
                output.bytes.extend_from_slice(&buffer[..retained]);
                if retained < bytes_read {
                    output.truncated = true;
                }
            }
            Err(error) if error.kind() == std::io::ErrorKind::Interrupted => continue,
            Err(_) => break,
        }
    }

    output
}

fn join_reader(reader: std::thread::JoinHandle<BoundedOutput>) -> BoundedOutput {
    reader.join().unwrap_or_default()
}

fn map_launch_failure(error: &std::io::Error) -> LaunchFailureKind {
    match error.kind() {
        std::io::ErrorKind::NotFound => LaunchFailureKind::NotFound,
        std::io::ErrorKind::PermissionDenied => LaunchFailureKind::PermissionDenied,
        _ => LaunchFailureKind::Other,
    }
}

#[derive(Clone, Copy, Debug, Default)]
pub struct SystemClock;

impl SystemClock {
    pub fn now_epoch_ms(&self) -> u64 {
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis() as u64
    }
}
