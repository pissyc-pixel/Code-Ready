use std::path::PathBuf;

use crate::domain::contracts::{PlatformId, ToolId};

pub mod fake;
pub mod fake_process;
pub mod native;
pub mod process;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum CandidateOrigin {
    Path,
    KnownLocation,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum CandidateKind {
    NativeBinary,
    NonNativeLauncher,
    AppleGitShim,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ExecutableCandidate {
    pub path: PathBuf,
    pub display_path: String,
    pub origin: CandidateOrigin,
    pub kind: CandidateKind,
}

impl ExecutableCandidate {
    pub fn native(
        path: impl Into<PathBuf>,
        display_path: impl Into<String>,
        origin: CandidateOrigin,
    ) -> Self {
        Self {
            path: path.into(),
            display_path: display_path.into(),
            origin,
            kind: CandidateKind::NativeBinary,
        }
    }

    pub fn new(
        path: impl Into<PathBuf>,
        display_path: impl Into<String>,
        origin: CandidateOrigin,
        kind: CandidateKind,
    ) -> Self {
        Self {
            path: path.into(),
            display_path: display_path.into(),
            origin,
            kind,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum PlatformProbeError {
    EnvironmentUnavailable,
    PathUnreadable,
}

pub trait PlatformAdapter: Send + Sync {
    fn platform(&self) -> PlatformId;
    fn tool_candidates(
        &self,
        tool_id: ToolId,
    ) -> Result<Vec<ExecutableCandidate>, PlatformProbeError>;
}
