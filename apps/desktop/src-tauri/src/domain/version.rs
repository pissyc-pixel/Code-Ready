use crate::domain::contracts::{ToolId, VersionStatus};

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct ParsedVersion {
    numeric: Vec<u64>,
    normalized: String,
}

impl ParsedVersion {
    pub fn parse_for(tool_id: ToolId, output: &[u8]) -> Option<Self> {
        let line = single_ascii_line(output)?;

        match tool_id {
            ToolId::Git => parse_git(line),
            ToolId::ClaudeCode => parse_claude(line),
            ToolId::CodexCli => parse_codex(line),
            ToolId::Winget | ToolId::Nodejs => None,
        }
    }

    pub fn normalized(&self) -> &str {
        &self.normalized
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum VersionPolicy {
    Unmanaged,
    Range {
        minimum_supported: ParsedVersion,
        newest_known: ParsedVersion,
    },
}

impl VersionPolicy {
    pub fn classify(&self, observed: &ParsedVersion) -> VersionStatus {
        match self {
            Self::Unmanaged => VersionStatus::NotComparable,
            Self::Range {
                minimum_supported,
                newest_known,
            } if observed < minimum_supported => VersionStatus::Outdated,
            Self::Range { newest_known, .. } if observed > newest_known => {
                VersionStatus::NewerThanKnown
            }
            Self::Range { .. } => VersionStatus::Current,
        }
    }
}

fn single_ascii_line(output: &[u8]) -> Option<&str> {
    if output.is_empty() || output.iter().any(|byte| *byte == 0 || !byte.is_ascii()) {
        return None;
    }

    let mut end = output.len();
    if output.last() == Some(&b'\n') {
        end -= 1;
        if end > 0 && output[end - 1] == b'\r' {
            end -= 1;
        }
    }

    let line = std::str::from_utf8(&output[..end]).ok()?;
    if line.is_empty() || line.contains(['\r', '\n']) {
        return None;
    }

    Some(line)
}

fn parse_git(line: &str) -> Option<ParsedVersion> {
    let token = line.strip_prefix("git version ")?;
    if let Some((numeric_token, apple_suffix)) = token.split_once(' ') {
        let numeric = parse_exact_three_numeric_segments(numeric_token)?;
        let build = apple_suffix
            .strip_prefix("(Apple Git-")?
            .strip_suffix(')')?;
        if build.is_empty() || !build.bytes().all(|byte| byte.is_ascii_digit()) {
            return None;
        }
        return Some(ParsedVersion {
            numeric,
            normalized: numeric_token.to_owned(),
        });
    }

    if token.is_empty() || token.contains(char::is_whitespace) {
        return None;
    }

    let numeric = parse_three_numeric_segments(token)?;
    let suffix = token.split('.').skip(3);
    if suffix.clone().any(|part| {
        part.is_empty()
            || !part
                .bytes()
                .all(|byte| byte.is_ascii_alphanumeric() || byte == b'-')
    }) {
        return None;
    }

    Some(ParsedVersion {
        numeric,
        normalized: token.to_owned(),
    })
}

fn parse_claude(line: &str) -> Option<ParsedVersion> {
    let token = line.strip_suffix(" (Claude Code)").unwrap_or(line);
    let numeric = parse_exact_three_numeric_segments(token)?;

    Some(ParsedVersion {
        numeric,
        normalized: token.to_owned(),
    })
}

fn parse_codex(line: &str) -> Option<ParsedVersion> {
    let token = line
        .strip_prefix("codex-cli ")
        .or_else(|| line.strip_prefix("codex "))?;
    if token.is_empty() || token.contains(char::is_whitespace) {
        return None;
    }

    let (numeric_token, suffix) = token.split_once('-').unwrap_or((token, ""));
    let numeric = parse_exact_three_numeric_segments(numeric_token)?;
    if !suffix.is_empty() {
        let (channel, number) = suffix.split_once('.')?;
        if !matches!(channel, "alpha" | "beta") || number.parse::<u64>().is_err() {
            return None;
        }
    }

    Some(ParsedVersion {
        numeric,
        normalized: token.to_owned(),
    })
}

fn parse_three_numeric_segments(token: &str) -> Option<Vec<u64>> {
    let mut segments = token.split('.');
    let numeric = (0..3)
        .map(|_| segments.next()?.parse::<u64>().ok())
        .collect::<Option<Vec<_>>>()?;
    Some(numeric)
}

fn parse_exact_three_numeric_segments(token: &str) -> Option<Vec<u64>> {
    let numeric = parse_three_numeric_segments(token)?;
    if token.split('.').count() == 3 {
        Some(numeric)
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::contracts::{ToolId, VersionStatus};

    #[test]
    fn parses_supported_native_cli_version_outputs() {
        assert_eq!(
            ParsedVersion::parse_for(ToolId::Git, b"git version 2.47.1.windows.1\n")
                .unwrap()
                .normalized(),
            "2.47.1.windows.1"
        );
        assert_eq!(
            ParsedVersion::parse_for(ToolId::ClaudeCode, b"2.1.89 (Claude Code)\n")
                .unwrap()
                .normalized(),
            "2.1.89"
        );
        assert_eq!(
            ParsedVersion::parse_for(ToolId::CodexCli, b"codex-cli 0.138.0\n")
                .unwrap()
                .normalized(),
            "0.138.0"
        );
    }

    #[test]
    fn parses_apple_git_version_suffix_as_the_numeric_version() {
        let parsed = ParsedVersion::parse_for(ToolId::Git, b"git version 2.39.3 (Apple Git-146)\n")
            .expect("Apple Git version is parseable");

        assert_eq!(parsed.normalized(), "2.39.3");
        assert_eq!(
            VersionPolicy::Unmanaged.classify(&parsed),
            VersionStatus::NotComparable
        );
    }

    #[test]
    fn rejects_unrelated_or_ambiguous_output() {
        for (tool, bytes) in [
            (ToolId::Git, b"welcome 2.47.1".as_slice()),
            (ToolId::ClaudeCode, b"Claude Code".as_slice()),
            (ToolId::CodexCli, b"codex 0.1.0 extra".as_slice()),
        ] {
            assert!(ParsedVersion::parse_for(tool, bytes).is_none());
        }
    }

    #[test]
    fn unmanaged_policy_never_claims_current() {
        let observed = ParsedVersion::parse_for(ToolId::CodexCli, b"codex-cli 0.138.0").unwrap();
        assert_eq!(
            VersionPolicy::Unmanaged.classify(&observed),
            VersionStatus::NotComparable
        );
    }
}
