use crate::domain::contracts::ToolId;

pub const DETECTABLE_TOOL_IDS: [ToolId; 3] = [ToolId::Git, ToolId::ClaudeCode, ToolId::CodexCli];

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum DetectionSelectionError {
    Empty,
    Duplicate(ToolId),
    Unsupported(ToolId),
}

pub fn validate_detection_selection(
    requested: Option<Vec<ToolId>>,
) -> Result<Vec<ToolId>, DetectionSelectionError> {
    let Some(requested) = requested else {
        return Ok(DETECTABLE_TOOL_IDS.to_vec());
    };

    if requested.is_empty() {
        return Err(DetectionSelectionError::Empty);
    }

    let mut validated = Vec::with_capacity(requested.len());
    for tool_id in requested {
        if !DETECTABLE_TOOL_IDS.contains(&tool_id) {
            return Err(DetectionSelectionError::Unsupported(tool_id));
        }
        if validated.contains(&tool_id) {
            return Err(DetectionSelectionError::Duplicate(tool_id));
        }
        validated.push(tool_id);
    }

    Ok(validated)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::contracts::ToolId;

    #[test]
    fn slice_one_selection_is_non_empty_unique_and_limited_to_three_detectors() {
        assert_eq!(
            validate_detection_selection(None).unwrap(),
            vec![ToolId::Git, ToolId::ClaudeCode, ToolId::CodexCli]
        );
        assert!(validate_detection_selection(Some(vec![])).is_err());
        assert!(validate_detection_selection(Some(vec![ToolId::Git, ToolId::Git])).is_err());
        assert!(validate_detection_selection(Some(vec![ToolId::Nodejs])).is_err());
        assert!(validate_detection_selection(Some(vec![ToolId::Winget])).is_err());
    }
}
