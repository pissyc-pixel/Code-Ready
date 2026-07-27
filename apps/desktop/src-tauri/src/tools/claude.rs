use crate::domain::contracts::{ToolId, ToolObservation};

use super::shared::{DetectionContext, ToolDetector, probe_native_version};

#[derive(Clone, Copy, Debug, Default)]
pub struct ClaudeDetector;

impl ToolDetector for ClaudeDetector {
    fn tool_id(&self) -> ToolId {
        ToolId::ClaudeCode
    }

    fn detect(&self, context: &DetectionContext<'_>) -> ToolObservation {
        probe_native_version(
            ToolId::ClaudeCode,
            context.platform,
            context.runner,
            context.clock,
        )
    }
}

pub fn boxed_claude_detector() -> Box<dyn ToolDetector> {
    Box::new(ClaudeDetector)
}

#[cfg(test)]
mod tests {}
