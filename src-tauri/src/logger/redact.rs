use std::sync::OnceLock;

use regex::Regex;

pub fn redact_line(line: &str) -> String {
    let mut redacted = bearer_regex().replace_all(line, "${prefix}[REDACTED]").into_owned();
    redacted = key_value_regex()
        .replace_all(&redacted, "${key}=[REDACTED]")
        .into_owned();
    sk_regex().replace_all(&redacted, "[REDACTED]").into_owned()
}

fn bearer_regex() -> &'static Regex {
    static REGEX: OnceLock<Regex> = OnceLock::new();
    REGEX.get_or_init(|| {
        Regex::new(r"(?i)(?P<prefix>authorization:\s*bearer\s+)\S+").expect("valid bearer regex")
    })
}

fn key_value_regex() -> &'static Regex {
    static REGEX: OnceLock<Regex> = OnceLock::new();
    REGEX.get_or_init(|| {
        Regex::new(r"(?i)\b(?P<key>token|api_key)\s*=\s*[^\s&]+").expect("valid key-value regex")
    })
}

fn sk_regex() -> &'static Regex {
    static REGEX: OnceLock<Regex> = OnceLock::new();
    REGEX.get_or_init(|| Regex::new(r"\bsk-[A-Za-z0-9_-]+\b").expect("valid sk regex"))
}

#[cfg(test)]
mod tests {
    use super::redact_line;

    #[test]
    fn redacts_bearer_tokens() {
        let redacted = redact_line("Authorization: Bearer secret-token-value");
        assert_eq!(redacted, "Authorization: Bearer [REDACTED]");
    }

    #[test]
    fn redacts_query_tokens_and_api_keys() {
        let redacted = redact_line("token=abc123 api_key=my-key sk-test-value");
        assert_eq!(redacted, "token=[REDACTED] api_key=[REDACTED] [REDACTED]");
    }
}
