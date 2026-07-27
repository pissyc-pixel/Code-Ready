use crate::domain::contracts::PlatformId;
use crate::domain::contracts::ToolId;

use super::{ExecutableCandidate, PlatformAdapter, PlatformProbeError};

#[derive(Clone, Debug)]
pub struct FakePlatformAdapter {
    platform: PlatformId,
    candidates: Vec<(ToolId, Result<Vec<ExecutableCandidate>, PlatformProbeError>)>,
}

impl FakePlatformAdapter {
    pub fn new(platform: PlatformId) -> Self {
        Self {
            platform,
            candidates: vec![],
        }
    }

    pub fn builder(platform: PlatformId) -> FakePlatformAdapterBuilder {
        FakePlatformAdapterBuilder {
            platform,
            candidates: vec![],
        }
    }
}

impl PlatformAdapter for FakePlatformAdapter {
    fn platform(&self) -> PlatformId {
        self.platform.clone()
    }

    fn tool_candidates(
        &self,
        tool_id: ToolId,
    ) -> Result<Vec<ExecutableCandidate>, PlatformProbeError> {
        self.candidates
            .iter()
            .find(|(configured_tool, _)| configured_tool == &tool_id)
            .map_or_else(|| Ok(vec![]), |(_, result)| result.clone())
    }
}

#[derive(Clone, Debug)]
pub struct FakePlatformAdapterBuilder {
    platform: PlatformId,
    candidates: Vec<(ToolId, Result<Vec<ExecutableCandidate>, PlatformProbeError>)>,
}

impl FakePlatformAdapterBuilder {
    pub fn with_candidates(
        mut self,
        tool_id: ToolId,
        candidates: Vec<ExecutableCandidate>,
    ) -> Self {
        self.candidates.push((tool_id, Ok(candidates)));
        self
    }

    pub fn with_probe_error(mut self, tool_id: ToolId, error: PlatformProbeError) -> Self {
        self.candidates.push((tool_id, Err(error)));
        self
    }

    pub fn build(self) -> FakePlatformAdapter {
        FakePlatformAdapter {
            platform: self.platform,
            candidates: self.candidates,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::contracts::ToolId;
    use crate::platform::CandidateOrigin;

    #[test]
    fn fake_adapter_returns_configured_candidates_without_host_cfg() {
        let claude = ExecutableCandidate::native(
            "/fake/home/.local/bin/claude",
            "~/.local/bin/claude",
            CandidateOrigin::KnownLocation,
        );
        let adapter = FakePlatformAdapter::builder(PlatformId::WindowsX64)
            .with_candidates(ToolId::ClaudeCode, vec![claude.clone()])
            .build();

        assert_eq!(
            adapter.tool_candidates(ToolId::ClaudeCode).unwrap(),
            vec![claude]
        );
        assert_eq!(adapter.platform(), PlatformId::WindowsX64);
    }
}
