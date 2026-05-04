use std::sync::OnceLock;

use regex::Regex;

pub fn redact_line(line: &str) -> String {
    let mut redacted = proxy_credentials_regex()
        .replace_all(line, "${scheme}[REDACTED]@")
        .into_owned();
    redacted = bearer_regex()
        .replace_all(&redacted, "${prefix}[REDACTED]")
        .into_owned();
    redacted = key_value_regex()
        .replace_all(&redacted, "${key}=[REDACTED]")
        .into_owned();
    sk_regex().replace_all(&redacted, "[REDACTED]").into_owned()
}

fn proxy_credentials_regex() -> &'static Regex {
    static REGEX: OnceLock<Regex> = OnceLock::new();
    REGEX.get_or_init(|| {
        Regex::new(r"(?i)(?P<scheme>\b(?:https?|socks5)://)[^/\s:@]+:[^/\s@]+@")
            .expect("valid proxy credentials regex")
    })
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
        Regex::new(r"(?i)\b(?P<key>openai_api_key|anthropic_api_key|gemini_api_key|api_key|token)\s*=\s*[^\s&]+")
            .expect("valid key-value regex")
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

    #[test]
    fn redacts_named_api_key_environment_values() {
        let line = "OPENAI_API_KEY=sk-openai ANTHROPIC_API_KEY=anthropic-secret GEMINI_API_KEY=gemini-secret API_KEY=generic-secret";
        let redacted = redact_line(line);

        for secret in [
            "sk-openai",
            "anthropic-secret",
            "gemini-secret",
            "generic-secret",
        ] {
            assert!(
                !redacted.contains(secret),
                "redacted output leaked {secret}: {redacted}"
            );
        }
        assert_eq!(
            redacted,
            "OPENAI_API_KEY=[REDACTED] ANTHROPIC_API_KEY=[REDACTED] GEMINI_API_KEY=[REDACTED] API_KEY=[REDACTED]"
        );
    }

    #[test]
    fn redacts_proxy_url_credentials() {
        let line = "http://user:pass@127.0.0.1:7890 https://user:pass@example.com socks5://user:pass@127.0.0.1:7890";
        let redacted = redact_line(line);

        assert!(!redacted.contains("user:pass"));
        assert_eq!(
            redacted,
            "http://[REDACTED]@127.0.0.1:7890 https://[REDACTED]@example.com socks5://[REDACTED]@127.0.0.1:7890"
        );
    }

    #[test]
    fn redacts_all_required_secret_shapes_without_leaking_originals() {
        let secrets = [
            "openai-secret",
            "anthropic-secret",
            "gemini-secret",
            "generic-secret",
            "bearer-secret",
            "sk-secret-value",
            "token-secret",
            "query-secret",
            "proxy-user:proxy-pass",
        ];
        let line = "OPENAI_API_KEY=openai-secret ANTHROPIC_API_KEY=anthropic-secret GEMINI_API_KEY=gemini-secret API_KEY=generic-secret Authorization: Bearer bearer-secret sk-secret-value token=token-secret api_key=query-secret http://proxy-user:proxy-pass@127.0.0.1:7890";

        let redacted = redact_line(line);

        for secret in secrets {
            assert!(
                !redacted.contains(secret),
                "redacted output leaked {secret}: {redacted}"
            );
        }
    }
}
