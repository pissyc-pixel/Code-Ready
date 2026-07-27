use crate::domain::contracts::{ToolId, ToolObservation};

use super::shared::{DetectionContext, ToolDetector, probe_native_version};

#[derive(Clone, Copy, Debug, Default)]
pub struct CodexDetector;

impl ToolDetector for CodexDetector {
    fn tool_id(&self) -> ToolId {
        ToolId::CodexCli
    }

    fn detect(&self, context: &DetectionContext<'_>) -> ToolObservation {
        probe_native_version(
            ToolId::CodexCli,
            context.platform,
            context.runner,
            context.clock,
        )
    }
}

pub fn boxed_codex_detector() -> Box<dyn ToolDetector> {
    Box::new(CodexDetector)
}

#[cfg(test)]
mod tests {}
