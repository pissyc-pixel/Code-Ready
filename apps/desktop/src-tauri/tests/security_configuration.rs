use std::fs;
use std::path::PathBuf;

use serde_json::Value;

fn read_json(path: &str) -> Value {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(path);
    let source = fs::read_to_string(&path).expect("security configuration exists");
    serde_json::from_str(&source).expect("security configuration is valid JSON")
}

#[test]
fn tauri_configuration_has_strict_csp_and_platform_bundle() {
    let config = read_json("tauri.conf.json");

    assert_eq!(config["identifier"], "com.codeready.v2");
    assert_eq!(config["app"]["windows"][0]["label"], "main");

    let csp = config["app"]["security"]["csp"]
        .as_str()
        .expect("CSP is a non-empty string");
    assert!(!csp.is_empty());
    assert!(csp.contains("default-src 'self'"));
    assert!(csp.contains("script-src 'self'"));
    assert!(!csp.contains("unsafe-eval"));
    assert!(!csp.contains("https:"));
    assert!(!csp.split_whitespace().any(|source| source == "*"));
    assert_eq!(
        csp.split(';')
            .map(str::trim)
            .find(|directive| directive.starts_with("connect-src")),
        Some("connect-src ipc: http://ipc.localhost")
    );

    assert_eq!(
        config["bundle"]["targets"],
        serde_json::json!(["app", "nsis"])
    );
    assert_eq!(config["bundle"]["macOS"]["minimumSystemVersion"], "13.0");
}

#[test]
fn main_capability_is_explicit_and_minimal() {
    let capability = read_json("capabilities/main.json");

    assert_eq!(capability["windows"], serde_json::json!(["main"]));
    assert_eq!(
        capability["permissions"],
        serde_json::json!([
            "core:app:default",
            "core:event:default",
            "core:window:default"
        ])
    );

    let serialized = capability["permissions"].to_string();
    for forbidden in [
        "shell", "process", "fs", "http", "dialog", "opener", "updater",
    ] {
        assert!(!serialized.contains(forbidden));
    }
}

#[test]
fn slice_one_adds_no_privileged_or_network_surface() {
    let capability = read_json("capabilities/main.json");
    assert_eq!(
        capability["permissions"],
        serde_json::json!([
            "core:app:default",
            "core:event:default",
            "core:window:default"
        ])
    );

    let cargo = read_text("Cargo.toml");
    for forbidden in ["reqwest", "ureq", "opentelemetry", "sentry", "analytics"] {
        assert!(!cargo.to_ascii_lowercase().contains(forbidden));
    }

    let config = read_json("tauri.conf.json");
    assert_eq!(
        config["app"]["security"]["csp"]
            .as_str()
            .unwrap()
            .split(';')
            .map(str::trim)
            .find(|part| part.starts_with("connect-src")),
        Some("connect-src ipc: http://ipc.localhost")
    );
}

#[test]
fn slice_one_keeps_the_ipc_event_dependency_and_generated_contract_surface_narrow() {
    let lib = read_text("src/lib.rs");
    assert!(lib.contains("api::commands::bootstrap"));
    assert!(lib.contains("api::commands::detect_tools"));
    assert_eq!(lib.matches("api::commands::").count(), 2);
    assert!(!lib.contains("get_bootstrap_state"));

    let events = read_text("src/api/events.rs");
    let production_events = events
        .split_once("#[cfg(test)]")
        .map_or(events.as_str(), |(production, _)| production);
    assert_eq!(
        production_events.matches("\"detection.changed\"").count(),
        1
    );

    let cargo = read_text("Cargo.toml").to_ascii_lowercase();
    for forbidden in [
        "reqwest",
        "ureq",
        "opentelemetry",
        "sentry",
        "analytics",
        "sqlite",
        "tokio",
        "async-std",
        "tauri-plugin-shell",
        "tauri-plugin-process",
    ] {
        assert!(!cargo.contains(forbidden), "Cargo.toml contains {forbidden}");
    }

    let package = read_text("../package.json").to_ascii_lowercase();
    for forbidden in ["i18n-http", "i18next-http", "telemetry", "analytics"] {
        assert!(!package.contains(forbidden), "package.json contains {forbidden}");
    }

    let generated = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../src/shared/api/generated");
    assert!(!generated.join("BootstrapState.ts").exists());
    assert!(generated.join("AppSnapshot.ts").exists());
    assert!(generated.join("DetectionEventEnvelope.ts").exists());
}

#[test]
fn ci_matrix_uses_both_compile_runners_without_claiming_clean_machine_acceptance() {
    let workflow_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../..")
        .join(".github/workflows/ci.yml");
    let workflow = fs::read_to_string(workflow_path).expect("CI workflow exists");

    for required in [
        "windows-2025",
        "macos-15",
        "node-version: 24",
        "MACOSX_DEPLOYMENT_TARGET: \"13.0\"",
        "npm ci",
        "npm run check",
        "npm run tauri:build",
        "uname -m",
        "arm64",
    ] {
        assert!(workflow.contains(required), "CI workflow lacks {required}");
    }

    assert!(!workflow.contains("Windows 11"));
}

fn read_text(path: &str) -> String {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(path);
    fs::read_to_string(&path).expect("configuration text exists")
}
