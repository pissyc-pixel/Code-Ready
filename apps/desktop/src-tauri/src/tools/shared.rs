use crate::domain::contracts::{
    DetectionEvidence, DetectionEvidenceCode, ObservedToolState, ProcessExitKind, ToolId,
    ToolObservation, VersionStatus,
};
use crate::domain::version::{ParsedVersion, VersionPolicy};
use crate::platform::process::{
    LaunchFailureKind, ProcessOutcome, ProcessRequest, ProcessRunner, SystemClock,
};
use crate::platform::{CandidateKind, CandidateOrigin, ExecutableCandidate, PlatformAdapter};

pub trait Clock: Send + Sync {
    fn now_epoch_ms(&self) -> u64;
}

impl Clock for SystemClock {
    fn now_epoch_ms(&self) -> u64 {
        self.now_epoch_ms()
    }
}

pub struct DetectionContext<'a> {
    pub platform: &'a dyn PlatformAdapter,
    pub runner: &'a dyn ProcessRunner,
    pub clock: &'a dyn Clock,
}

pub trait ToolDetector: Send + Sync {
    fn tool_id(&self) -> ToolId;
    fn detect(&self, context: &DetectionContext<'_>) -> ToolObservation;
}

pub fn probe_native_version(
    tool_id: ToolId,
    platform: &dyn PlatformAdapter,
    runner: &dyn ProcessRunner,
    clock: &dyn Clock,
) -> ToolObservation {
    let checked_at_epoch_ms = clock.now_epoch_ms();
    let candidates = match platform.tool_candidates(tool_id.clone()) {
        Ok(candidates) => candidates,
        Err(_) => {
            return observation(
                tool_id,
                ObservedToolState::Unknown,
                None,
                VersionStatus::Unknown,
                DetectionEvidence {
                    code: DetectionEvidenceCode::PlatformProbeFailed,
                    display_path: None,
                    exit: None,
                },
                checked_at_epoch_ms,
            );
        }
    };

    probe_native_candidates(tool_id, candidates, runner, checked_at_epoch_ms)
}

pub(crate) fn probe_native_candidates(
    tool_id: ToolId,
    candidates: Vec<ExecutableCandidate>,
    runner: &dyn ProcessRunner,
    checked_at_epoch_ms: u64,
) -> ToolObservation {
    let Some(selection) = select_candidate(&candidates) else {
        return observation(
            tool_id,
            ObservedToolState::Absent,
            None,
            VersionStatus::Unknown,
            DetectionEvidence {
                code: DetectionEvidenceCode::NotFound,
                display_path: None,
                exit: None,
            },
            checked_at_epoch_ms,
        );
    };

    let Some(candidate) = selection.candidate else {
        return observation(
            tool_id,
            ObservedToolState::PresentBroken,
            None,
            VersionStatus::Unknown,
            DetectionEvidence {
                code: DetectionEvidenceCode::NonNativeLauncher,
                display_path: Some(selection.display_path),
                exit: None,
            },
            checked_at_epoch_ms,
        );
    };

    let outcome = runner.run(ProcessRequest::version_probe(candidate.path.clone()));
    let display_path = Some(candidate.display_path.clone());
    let base_state = selection.state;
    let base_code = selection.evidence_code;

    match outcome {
        ProcessOutcome::Exited {
            code: Some(0),
            stdout,
            stderr,
            ..
        } => {
            let parsed = ParsedVersion::parse_for(tool_id.clone(), &stdout)
                .or_else(|| ParsedVersion::parse_for(tool_id.clone(), &stderr));
            let (version, version_status, evidence_code) = match parsed {
                Some(parsed) => (
                    Some(parsed.normalized().to_owned()),
                    VersionPolicy::Unmanaged.classify(&parsed),
                    base_code,
                ),
                None => (
                    None,
                    VersionStatus::Unknown,
                    DetectionEvidenceCode::VersionUnparseable,
                ),
            };
            observation(
                tool_id,
                base_state,
                version,
                version_status,
                DetectionEvidence {
                    code: evidence_code,
                    display_path,
                    exit: Some(ProcessExitKind::Success),
                },
                checked_at_epoch_ms,
            )
        }
        ProcessOutcome::Exited { code, .. } => observation(
            tool_id,
            ObservedToolState::PresentBroken,
            None,
            VersionStatus::Unknown,
            DetectionEvidence {
                code: DetectionEvidenceCode::CommandNonZero,
                display_path,
                exit: Some(if code.is_some() {
                    ProcessExitKind::NonZero
                } else {
                    ProcessExitKind::Signalled
                }),
            },
            checked_at_epoch_ms,
        ),
        ProcessOutcome::TimedOut => observation(
            tool_id,
            ObservedToolState::PresentBroken,
            None,
            VersionStatus::Unknown,
            DetectionEvidence {
                code: DetectionEvidenceCode::CommandTimedOut,
                display_path,
                exit: Some(ProcessExitKind::TimedOut),
            },
            checked_at_epoch_ms,
        ),
        ProcessOutcome::LaunchFailed(kind) => {
            let state = if kind == LaunchFailureKind::NotFound {
                ObservedToolState::Unknown
            } else {
                ObservedToolState::PresentBroken
            };
            observation(
                tool_id,
                state,
                None,
                VersionStatus::Unknown,
                DetectionEvidence {
                    code: DetectionEvidenceCode::CommandLaunchFailed,
                    display_path,
                    exit: Some(ProcessExitKind::LaunchFailed),
                },
                checked_at_epoch_ms,
            )
        }
    }
}

struct CandidateSelection<'a> {
    candidate: Option<&'a ExecutableCandidate>,
    state: ObservedToolState,
    evidence_code: DetectionEvidenceCode,
    display_path: String,
}

fn select_candidate(candidates: &[ExecutableCandidate]) -> Option<CandidateSelection<'_>> {
    let path_candidate = candidates
        .iter()
        .find(|candidate| candidate.origin == CandidateOrigin::Path);
    let known_native = candidates.iter().find(|candidate| {
        candidate.origin == CandidateOrigin::KnownLocation
            && candidate.kind == CandidateKind::NativeBinary
    });

    match path_candidate {
        Some(candidate) if candidate.kind == CandidateKind::NativeBinary => {
            Some(CandidateSelection {
                candidate: Some(candidate),
                state: ObservedToolState::PresentHealthy,
                evidence_code: DetectionEvidenceCode::PathCommandHealthy,
                display_path: candidate.display_path.clone(),
            })
        }
        Some(_candidate) if known_native.is_some() => {
            let known_native = known_native.expect("checked above");
            Some(CandidateSelection {
                candidate: Some(known_native),
                state: ObservedToolState::PresentPathIssue,
                evidence_code: DetectionEvidenceCode::PathShadowed,
                display_path: known_native.display_path.clone(),
            })
        }
        Some(candidate) => Some(CandidateSelection {
            candidate: None,
            state: ObservedToolState::PresentBroken,
            evidence_code: DetectionEvidenceCode::NonNativeLauncher,
            display_path: candidate.display_path.clone(),
        }),
        None => {
            let known = candidates
                .iter()
                .find(|candidate| candidate.origin == CandidateOrigin::KnownLocation);
            match known {
                Some(candidate) if candidate.kind == CandidateKind::NativeBinary => {
                    Some(CandidateSelection {
                        candidate: Some(candidate),
                        state: ObservedToolState::PresentPathIssue,
                        evidence_code: DetectionEvidenceCode::KnownLocationHealthy,
                        display_path: candidate.display_path.clone(),
                    })
                }
                Some(candidate) => Some(CandidateSelection {
                    candidate: None,
                    state: ObservedToolState::PresentBroken,
                    evidence_code: DetectionEvidenceCode::NonNativeLauncher,
                    display_path: candidate.display_path.clone(),
                }),
                None => None,
            }
        }
    }
}

pub(crate) fn observation(
    tool_id: ToolId,
    state: ObservedToolState,
    version: Option<String>,
    version_status: VersionStatus,
    evidence: DetectionEvidence,
    checked_at_epoch_ms: u64,
) -> ToolObservation {
    ToolObservation {
        tool_id,
        state,
        version,
        version_status,
        evidence,
        checked_at_epoch_ms,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::contracts::{
        DetectionEvidenceCode, ObservedToolState, PlatformId, ToolId, ToolObservation,
        VersionStatus,
    };
    use crate::platform::fake::FakePlatformAdapter;
    use crate::platform::fake_process::FakeProcessRunner;
    use crate::platform::process::{LaunchFailureKind, ProcessOutcome};
    use crate::platform::{
        CandidateKind, CandidateOrigin, ExecutableCandidate, PlatformProbeError,
    };

    struct Case {
        name: &'static str,
        candidates: Result<Vec<ExecutableCandidate>, PlatformProbeError>,
        outcomes: Vec<ProcessOutcome>,
        expected_state: ObservedToolState,
        expected_version: Option<&'static str>,
        expected_version_status: VersionStatus,
        expected_evidence: DetectionEvidenceCode,
    }

    struct FixedClock;

    impl Clock for FixedClock {
        fn now_epoch_ms(&self) -> u64 {
            1_754_000_000_000
        }
    }

    fn native_path(path: &str) -> ExecutableCandidate {
        ExecutableCandidate::native(path, path, CandidateOrigin::Path)
    }

    fn native_known(path: &str) -> ExecutableCandidate {
        ExecutableCandidate::native(path, path, CandidateOrigin::KnownLocation)
    }

    fn non_native_path(path: &str) -> ExecutableCandidate {
        ExecutableCandidate::new(
            path,
            path,
            CandidateOrigin::Path,
            CandidateKind::NonNativeLauncher,
        )
    }

    fn success(output: &str) -> ProcessOutcome {
        ProcessOutcome::Exited {
            code: Some(0),
            stdout: output.as_bytes().to_vec(),
            stderr: vec![],
            stdout_truncated: false,
            stderr_truncated: false,
        }
    }

    fn nonzero(code: i32) -> ProcessOutcome {
        ProcessOutcome::Exited {
            code: Some(code),
            stdout: vec![],
            stderr: vec![],
            stdout_truncated: false,
            stderr_truncated: false,
        }
    }

    fn run_case(platform: PlatformId, case: &Case) -> ToolObservation {
        let adapter = match case.candidates.clone() {
            Ok(candidates) => FakePlatformAdapter::builder(platform)
                .with_candidates(ToolId::ClaudeCode, candidates)
                .build(),
            Err(error) => FakePlatformAdapter::builder(platform)
                .with_probe_error(ToolId::ClaudeCode, error)
                .build(),
        };
        let runner = FakeProcessRunner::new(case.outcomes.clone());
        let clock = FixedClock;
        probe_native_version(ToolId::ClaudeCode, &adapter, &runner, &clock)
    }

    #[test]
    fn native_cli_fact_matrix_is_platform_independent() {
        let cases = [
            Case {
                name: "path native success",
                candidates: Ok(vec![native_path("/path/claude")]),
                outcomes: vec![success("2.1.89 (Claude Code)\n")],
                expected_state: ObservedToolState::PresentHealthy,
                expected_version: Some("2.1.89"),
                expected_version_status: VersionStatus::NotComparable,
                expected_evidence: DetectionEvidenceCode::PathCommandHealthy,
            },
            Case {
                name: "known native success",
                candidates: Ok(vec![native_known("/home/.local/bin/claude")]),
                outcomes: vec![success("2.1.89\n")],
                expected_state: ObservedToolState::PresentPathIssue,
                expected_version: Some("2.1.89"),
                expected_version_status: VersionStatus::NotComparable,
                expected_evidence: DetectionEvidenceCode::KnownLocationHealthy,
            },
            Case {
                name: "not found",
                candidates: Ok(vec![]),
                outcomes: vec![],
                expected_state: ObservedToolState::Absent,
                expected_version: None,
                expected_version_status: VersionStatus::Unknown,
                expected_evidence: DetectionEvidenceCode::NotFound,
            },
            Case {
                name: "nonzero",
                candidates: Ok(vec![native_path("/path/claude")]),
                outcomes: vec![nonzero(2)],
                expected_state: ObservedToolState::PresentBroken,
                expected_version: None,
                expected_version_status: VersionStatus::Unknown,
                expected_evidence: DetectionEvidenceCode::CommandNonZero,
            },
            Case {
                name: "timeout",
                candidates: Ok(vec![native_path("/path/claude")]),
                outcomes: vec![ProcessOutcome::TimedOut],
                expected_state: ObservedToolState::PresentBroken,
                expected_version: None,
                expected_version_status: VersionStatus::Unknown,
                expected_evidence: DetectionEvidenceCode::CommandTimedOut,
            },
            Case {
                name: "signal",
                candidates: Ok(vec![native_path("/path/claude")]),
                outcomes: vec![ProcessOutcome::Exited {
                    code: None,
                    stdout: vec![],
                    stderr: vec![],
                    stdout_truncated: false,
                    stderr_truncated: false,
                }],
                expected_state: ObservedToolState::PresentBroken,
                expected_version: None,
                expected_version_status: VersionStatus::Unknown,
                expected_evidence: DetectionEvidenceCode::CommandNonZero,
            },
            Case {
                name: "permission denied",
                candidates: Ok(vec![native_path("/path/claude")]),
                outcomes: vec![ProcessOutcome::LaunchFailed(
                    LaunchFailureKind::PermissionDenied,
                )],
                expected_state: ObservedToolState::PresentBroken,
                expected_version: None,
                expected_version_status: VersionStatus::Unknown,
                expected_evidence: DetectionEvidenceCode::CommandLaunchFailed,
            },
            Case {
                name: "candidate disappeared",
                candidates: Ok(vec![native_path("/path/claude")]),
                outcomes: vec![ProcessOutcome::LaunchFailed(LaunchFailureKind::NotFound)],
                expected_state: ObservedToolState::Unknown,
                expected_version: None,
                expected_version_status: VersionStatus::Unknown,
                expected_evidence: DetectionEvidenceCode::CommandLaunchFailed,
            },
            Case {
                name: "script only",
                candidates: Ok(vec![non_native_path("/path/claude")]),
                outcomes: vec![],
                expected_state: ObservedToolState::PresentBroken,
                expected_version: None,
                expected_version_status: VersionStatus::Unknown,
                expected_evidence: DetectionEvidenceCode::NonNativeLauncher,
            },
            Case {
                name: "platform failure",
                candidates: Err(PlatformProbeError::PathUnreadable),
                outcomes: vec![],
                expected_state: ObservedToolState::Unknown,
                expected_version: None,
                expected_version_status: VersionStatus::Unknown,
                expected_evidence: DetectionEvidenceCode::PlatformProbeFailed,
            },
        ];

        for platform in [PlatformId::WindowsX64, PlatformId::MacosArm64] {
            for case in &cases {
                let observation = run_case(platform.clone(), case);
                assert_eq!(observation.state, case.expected_state, "{}", case.name);
                assert_eq!(
                    observation.version.as_deref(),
                    case.expected_version,
                    "{}",
                    case.name
                );
                assert_eq!(
                    observation.version_status, case.expected_version_status,
                    "{}",
                    case.name
                );
                assert_eq!(
                    observation.evidence.code, case.expected_evidence,
                    "{}",
                    case.name
                );
            }
        }
    }

    #[test]
    fn exit_zero_with_unparseable_version_keeps_tool_health_separate() {
        let adapter = FakePlatformAdapter::builder(PlatformId::MacosArm64)
            .with_candidates(ToolId::ClaudeCode, vec![native_path("/path/claude")])
            .build();
        let runner = FakeProcessRunner::new([success("Claude Code\n")]);
        let clock = FixedClock;
        let observation = probe_native_version(ToolId::ClaudeCode, &adapter, &runner, &clock);

        assert_eq!(observation.state, ObservedToolState::PresentHealthy);
        assert_eq!(observation.version, None);
        assert_eq!(observation.version_status, VersionStatus::Unknown);
        assert_eq!(
            observation.evidence.code,
            DetectionEvidenceCode::VersionUnparseable
        );
    }

    #[test]
    fn non_native_launcher_is_never_executed() {
        let runner = FakeProcessRunner::new([success("2.1.89\n")]);
        let adapter = FakePlatformAdapter::builder(PlatformId::MacosArm64)
            .with_candidates(
                ToolId::ClaudeCode,
                vec![
                    non_native_path("/path/claude"),
                    native_known("/home/.local/bin/claude"),
                ],
            )
            .build();
        let clock = FixedClock;
        let observation = probe_native_version(ToolId::ClaudeCode, &adapter, &runner, &clock);

        assert_eq!(runner.requests().len(), 1);
        assert_eq!(
            runner.requests()[0].executable,
            std::path::PathBuf::from("/home/.local/bin/claude")
        );
        assert_eq!(observation.state, ObservedToolState::PresentPathIssue);
        assert_eq!(
            observation.evidence.code,
            DetectionEvidenceCode::PathShadowed
        );
    }
}
