use std::path::Path;

use crate::domain::contracts::{
    DetectionEvidence, DetectionEvidenceCode, ObservedToolState, ProcessExitKind, ToolId,
    VersionStatus,
};
use crate::platform::process::{ProcessOutcome, ProcessRequest};
use crate::platform::{CandidateKind, CandidateOrigin, ExecutableCandidate};

use super::shared::{
    DetectionContext, ToolDetector, observation, probe_native_candidates, probe_native_version,
};

#[derive(Clone, Copy, Debug, Default)]
pub struct GitDetector;

impl ToolDetector for GitDetector {
    fn tool_id(&self) -> ToolId {
        ToolId::Git
    }

    fn detect(&self, context: &DetectionContext<'_>) -> crate::domain::contracts::ToolObservation {
        let checked_at_epoch_ms = context.clock.now_epoch_ms();
        let candidates = match context.platform.tool_candidates(ToolId::Git) {
            Ok(candidates) => candidates,
            Err(_) => {
                return probe_native_version(
                    ToolId::Git,
                    context.platform,
                    context.runner,
                    context.clock,
                );
            }
        };

        let non_apple_candidates = candidates
            .iter()
            .filter(|candidate| candidate.kind != CandidateKind::AppleGitShim)
            .cloned()
            .collect::<Vec<_>>();
        if !non_apple_candidates.is_empty() || candidates.is_empty() {
            return probe_native_candidates(
                ToolId::Git,
                non_apple_candidates,
                context.runner,
                checked_at_epoch_ms,
            );
        }

        let display_path = candidates
            .first()
            .map(|candidate| candidate.display_path.clone())
            .unwrap_or_else(|| "/usr/bin/git".to_owned());
        match context
            .runner
            .run(ProcessRequest::apple_developer_dir_probe())
        {
            ProcessOutcome::Exited {
                code: Some(0),
                stdout,
                ..
            } if parse_developer_directory(&stdout).is_some() => {
                let shim = ExecutableCandidate::new(
                    "/usr/bin/git",
                    display_path,
                    CandidateOrigin::Path,
                    CandidateKind::NativeBinary,
                );
                probe_native_candidates(
                    ToolId::Git,
                    vec![shim],
                    context.runner,
                    checked_at_epoch_ms,
                )
            }
            ProcessOutcome::Exited { code: Some(0), .. } => {
                platform_probe_failed(display_path, ProcessExitKind::Success, checked_at_epoch_ms)
            }
            ProcessOutcome::Exited { code: Some(_), .. } => observation(
                ToolId::Git,
                ObservedToolState::Absent,
                None,
                VersionStatus::Unknown,
                DetectionEvidence {
                    code: DetectionEvidenceCode::AppleDeveloperToolsMissing,
                    display_path: Some(display_path),
                    exit: Some(ProcessExitKind::NonZero),
                },
                checked_at_epoch_ms,
            ),
            ProcessOutcome::Exited { code: None, .. } => platform_probe_failed(
                display_path,
                ProcessExitKind::Signalled,
                checked_at_epoch_ms,
            ),
            ProcessOutcome::TimedOut => {
                platform_probe_failed(display_path, ProcessExitKind::TimedOut, checked_at_epoch_ms)
            }
            ProcessOutcome::LaunchFailed(_) => platform_probe_failed(
                display_path,
                ProcessExitKind::LaunchFailed,
                checked_at_epoch_ms,
            ),
        }
    }
}

pub fn boxed_git_detector() -> Box<dyn ToolDetector> {
    Box::new(GitDetector)
}

fn parse_developer_directory(output: &[u8]) -> Option<&str> {
    if output.is_empty() || output.iter().any(|byte| *byte == 0 || !byte.is_ascii()) {
        return None;
    }
    let mut text = std::str::from_utf8(output).ok()?;
    if text.ends_with('\n') {
        text = &text[..text.len() - 1];
        if text.ends_with('\r') {
            text = &text[..text.len() - 1];
        }
    }
    if text.is_empty() || text.contains(['\r', '\n']) || !Path::new(text).is_absolute() {
        return None;
    }
    Some(text)
}

fn platform_probe_failed(
    display_path: String,
    exit: ProcessExitKind,
    checked_at_epoch_ms: u64,
) -> crate::domain::contracts::ToolObservation {
    observation(
        ToolId::Git,
        ObservedToolState::Unknown,
        None,
        VersionStatus::Unknown,
        DetectionEvidence {
            code: DetectionEvidenceCode::PlatformProbeFailed,
            display_path: Some(display_path),
            exit: Some(exit),
        },
        checked_at_epoch_ms,
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::contracts::PlatformId;
    use crate::platform::fake::FakePlatformAdapter;
    use crate::platform::fake_process::FakeProcessRunner;
    use crate::platform::process::{ProcessOutcome, SystemClock};

    #[test]
    fn apple_git_preflight_accepts_a_developer_directory_with_spaces_before_git_probe() {
        let adapter = FakePlatformAdapter::builder(PlatformId::MacosArm64)
            .with_candidates(
                ToolId::Git,
                vec![ExecutableCandidate::new(
                    "/usr/bin/git",
                    "/usr/bin/git",
                    CandidateOrigin::Path,
                    CandidateKind::AppleGitShim,
                )],
            )
            .build();
        let runner = FakeProcessRunner::new([
            ProcessOutcome::Exited {
                code: Some(0),
                stdout: b"/Applications/Xcode 15.app/Contents/Developer\n".to_vec(),
                stderr: vec![],
                stdout_truncated: false,
                stderr_truncated: false,
            },
            ProcessOutcome::Exited {
                code: Some(0),
                stdout: b"git version 2.47.1\n".to_vec(),
                stderr: vec![],
                stdout_truncated: false,
                stderr_truncated: false,
            },
        ]);
        let clock = SystemClock;
        let context = DetectionContext {
            platform: &adapter,
            runner: &runner,
            clock: &clock,
        };

        let observation = GitDetector.detect(&context);

        assert_eq!(observation.state, ObservedToolState::PresentHealthy);
        assert_eq!(observation.version.as_deref(), Some("2.47.1"));
        assert_eq!(runner.requests().len(), 2);
        assert_eq!(
            runner.requests()[0],
            ProcessRequest::apple_developer_dir_probe()
        );
        assert_eq!(
            runner.requests()[1],
            ProcessRequest::version_probe("/usr/bin/git")
        );
    }

    #[test]
    fn apple_git_missing_developer_tools_never_runs_the_git_shim() {
        let adapter = FakePlatformAdapter::builder(PlatformId::MacosArm64)
            .with_candidates(
                ToolId::Git,
                vec![ExecutableCandidate::new(
                    "/usr/bin/git",
                    "/usr/bin/git",
                    CandidateOrigin::Path,
                    CandidateKind::AppleGitShim,
                )],
            )
            .build();
        let runner = FakeProcessRunner::new([ProcessOutcome::Exited {
            code: Some(1),
            stdout: vec![],
            stderr: b"xcode-select: error: toolchain path is not set\n".to_vec(),
            stdout_truncated: false,
            stderr_truncated: false,
        }]);
        let clock = SystemClock;
        let context = DetectionContext {
            platform: &adapter,
            runner: &runner,
            clock: &clock,
        };

        let observation = GitDetector.detect(&context);

        assert_eq!(observation.state, ObservedToolState::Absent);
        assert_eq!(
            observation.evidence.code,
            DetectionEvidenceCode::AppleDeveloperToolsMissing
        );
        assert_eq!(
            runner.requests(),
            vec![ProcessRequest::apple_developer_dir_probe()]
        );
    }

    #[test]
    fn apple_git_invalid_successful_preflight_is_unknown_and_never_runs_the_shim() {
        let adapter = FakePlatformAdapter::builder(PlatformId::MacosArm64)
            .with_candidates(
                ToolId::Git,
                vec![ExecutableCandidate::new(
                    "/usr/bin/git",
                    "/usr/bin/git",
                    CandidateOrigin::Path,
                    CandidateKind::AppleGitShim,
                )],
            )
            .build();
        let runner = FakeProcessRunner::new([ProcessOutcome::Exited {
            code: Some(0),
            stdout: b"not-a-developer-directory\n".to_vec(),
            stderr: vec![],
            stdout_truncated: false,
            stderr_truncated: false,
        }]);
        let clock = SystemClock;
        let context = DetectionContext {
            platform: &adapter,
            runner: &runner,
            clock: &clock,
        };

        let observation = GitDetector.detect(&context);

        assert_eq!(observation.state, ObservedToolState::Unknown);
        assert_eq!(
            observation.evidence.code,
            DetectionEvidenceCode::PlatformProbeFailed
        );
        assert_eq!(
            runner.requests(),
            vec![ProcessRequest::apple_developer_dir_probe()]
        );
    }
}
