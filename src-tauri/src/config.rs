use std::{
    fs,
    path::PathBuf,
};

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum InstallNetworkMode {
    None,
    SystemProxy,
    ManualProxy,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum NpmRegistryOption {
    Default,
    Npmmirror,
    Custom,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct InstallNetworkConfig {
    pub mode: InstallNetworkMode,
    pub proxy_url: Option<String>,
    pub npm_registry: NpmRegistryOption,
    pub custom_npm_registry: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum CcSwitchDownloadSourceKind {
    DirectExe,
    DirectZip,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct CcSwitchDownloadSource {
    pub name: String,
    pub url: String,
    pub priority: i32,
    pub enabled: bool,
    pub kind: CcSwitchDownloadSourceKind,
    pub sha256: Option<String>,
    pub min_file_size_bytes: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AppConfig {
    pub install_network: InstallNetworkConfig,
    #[serde(default)]
    pub ccswitch_path: Option<String>,
    #[serde(default)]
    pub ccswitch_download_sources: Vec<CcSwitchDownloadSource>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct InstallNetworkPatch {
    pub mode: Option<InstallNetworkMode>,
    pub proxy_url: Option<String>,
    pub npm_registry: Option<NpmRegistryOption>,
    pub custom_npm_registry: Option<String>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AppConfigPatch {
    pub install_network: Option<InstallNetworkPatch>,
    pub ccswitch_path: Option<String>,
    pub ccswitch_download_sources: Option<Vec<CcSwitchDownloadSource>>,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            install_network: InstallNetworkConfig {
                mode: InstallNetworkMode::None,
                proxy_url: None,
                npm_registry: NpmRegistryOption::Default,
                custom_npm_registry: None,
            },
            ccswitch_path: None,
            ccswitch_download_sources: Vec::new(),
        }
    }
}

pub fn get_config() -> Result<AppConfig, String> {
    let path = config_path()?;
    get_config_from_path(&path)
}

fn get_config_from_path(path: &std::path::Path) -> Result<AppConfig, String> {
    if !path.exists() {
        let config = AppConfig::default();
        write_config_to_path(path, &config)?;
        return Ok(config);
    }

    let content = fs::read_to_string(&path).map_err(|error| error.to_string())?;
    match serde_json::from_str::<AppConfig>(&content) {
        Ok(config) => {
            let config = sanitize_config(config);
            validate_config(&config)?;
            write_config_to_path(path, &config)?;
            Ok(config)
        }
        Err(_) => {
            backup_corrupt_config(path)?;
            let config = AppConfig::default();
            write_config_to_path(path, &config)?;
            Ok(config)
        }
    }
}

pub fn update_config(patch: AppConfigPatch) -> Result<AppConfig, String> {
    let path = config_path()?;
    update_config_at_path(&path, patch)
}

fn update_config_at_path(
    path: &std::path::Path,
    patch: AppConfigPatch,
) -> Result<AppConfig, String> {
    let mut config = get_config_from_path(path)?;
    if let Some(network_patch) = patch.install_network {
        if let Some(mode) = network_patch.mode {
            config.install_network.mode = mode;
        }
        if let Some(proxy_url) = network_patch.proxy_url {
            config.install_network.proxy_url = Some(proxy_url);
        }
        if let Some(npm_registry) = network_patch.npm_registry {
            config.install_network.npm_registry = npm_registry;
        }
        if let Some(custom_npm_registry) = network_patch.custom_npm_registry {
            config.install_network.custom_npm_registry = Some(custom_npm_registry);
        }
    }
    if let Some(ccswitch_path) = patch.ccswitch_path {
        let trimmed = ccswitch_path.trim();
        config.ccswitch_path = if trimmed.is_empty() {
            None
        } else {
            Some(trimmed.to_string())
        };
    }
    if let Some(ccswitch_download_sources) = patch.ccswitch_download_sources {
        config.ccswitch_download_sources = ccswitch_download_sources;
    }
    config = sanitize_config(config);
    validate_config(&config)?;
    write_config_to_path(path, &config)?;
    Ok(config)
}

pub fn reset_config() -> Result<AppConfig, String> {
    let config = AppConfig::default();
    write_config(&config)?;
    Ok(config)
}

fn write_config(config: &AppConfig) -> Result<(), String> {
    let path = config_path()?;
    write_config_to_path(&path, config)
}

fn write_config_to_path(path: &std::path::Path, config: &AppConfig) -> Result<(), String> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|error| error.to_string())?;
    }
    let content = serde_json::to_string_pretty(config).map_err(|error| error.to_string())?;
    fs::write(path, content).map_err(|error| error.to_string())
}

fn config_path() -> Result<PathBuf, String> {
    let appdata = std::env::var_os("APPDATA")
        .map(PathBuf::from)
        .ok_or_else(|| "APPDATA is not available".to_string())?;
    Ok(appdata.join("ai-coding-installer").join("config.json"))
}

fn backup_corrupt_config(path: &std::path::Path) -> Result<(), String> {
    let parent = path
        .parent()
        .ok_or_else(|| "config path has no parent directory".to_string())?;
    let backup_name = format!(
        "config.bak.{}.json",
        chrono::Local::now().format("%Y%m%d-%H%M%S")
    );
    fs::copy(path, parent.join(backup_name))
        .map(|_| ())
        .map_err(|error| error.to_string())
}

fn sanitize_config(mut config: AppConfig) -> AppConfig {
    config.install_network.proxy_url = trim_optional(config.install_network.proxy_url);
    config.install_network.custom_npm_registry =
        trim_optional(config.install_network.custom_npm_registry);
    config.ccswitch_path = trim_optional(config.ccswitch_path);
    config.ccswitch_download_sources = config
        .ccswitch_download_sources
        .into_iter()
        .map(|mut source| {
            source.name = source.name.trim().to_string();
            source.url = source.url.trim().to_string();
            source.sha256 = trim_optional(source.sha256);
            source
        })
        .collect();
    config
}

fn trim_optional(value: Option<String>) -> Option<String> {
    value.and_then(|value| {
        let trimmed = value.trim();
        if trimmed.is_empty() {
            None
        } else {
            Some(trimmed.to_string())
        }
    })
}

fn validate_config(config: &AppConfig) -> Result<(), String> {
    if let Some(url) = &config.install_network.custom_npm_registry {
        validate_http_url(url, "custom npm registry")?;
    }
    if matches!(config.install_network.npm_registry, NpmRegistryOption::Custom)
        && config.install_network.custom_npm_registry.is_none()
    {
        return Err("custom npm registry requires a valid http:// or https:// URL".to_string());
    }
    for source in &config.ccswitch_download_sources {
        validate_http_url(&source.url, "ccSwitch download source")?;
    }
    Ok(())
}

fn validate_http_url(value: &str, label: &str) -> Result<(), String> {
    if value.chars().any(char::is_whitespace) {
        return Err(format!("{label} URL must not contain whitespace"));
    }
    let (scheme, rest) = value
        .split_once("://")
        .ok_or_else(|| format!("{label} must be a valid http:// or https:// URL"))?;
    if !matches!(scheme.to_ascii_lowercase().as_str(), "http" | "https") {
        return Err(format!("{label} must use http:// or https://"));
    }
    let authority = rest
        .split(['/', '?', '#'])
        .next()
        .unwrap_or_default();
    if authority.is_empty() {
        return Err(format!("{label} URL must include a host"));
    }
    if authority.contains('@') {
        return Err(format!("{label} URL must not include credentials"));
    }
    let host = if let Some(stripped) = authority.strip_prefix('[') {
        stripped
            .split_once(']')
            .map(|(host, _)| host)
            .unwrap_or_default()
    } else {
        authority.split(':').next().unwrap_or_default()
    };
    if host.is_empty() {
        return Err(format!("{label} URL must include a host"));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_config_omits_unused_pip_index_mode() {
        let json = serde_json::to_value(AppConfig::default()).expect("serialize config");
        assert!(json.get("pipIndexMode").is_none());
        assert_eq!(json["installNetwork"]["mode"], "none");
        assert!(json.get("ccswitchPath").is_some());
        assert_eq!(json["ccswitchDownloadSources"], serde_json::json!([]));
    }

    #[test]
    fn merges_config_patch() {
        let config = AppConfig::default();
        let patch = AppConfigPatch {
            install_network: Some(InstallNetworkPatch {
                mode: Some(InstallNetworkMode::ManualProxy),
                proxy_url: Some("http://127.0.0.1:7890".to_string()),
                npm_registry: Some(NpmRegistryOption::Npmmirror),
                custom_npm_registry: None,
            }),
            ccswitch_path: Some("C:\\Program Files\\ccswitch\\ccswitch.exe".to_string()),
            ccswitch_download_sources: Some(vec![CcSwitchDownloadSource {
                name: "Official TODO".to_string(),
                url: "https://example.invalid/ccswitch.exe".to_string(),
                priority: 10,
                enabled: true,
                kind: CcSwitchDownloadSourceKind::DirectExe,
                sha256: None,
                min_file_size_bytes: Some(1024),
            }]),
        };

        let mut merged = config;
        if let Some(network_patch) = patch.install_network {
            if let Some(mode) = network_patch.mode {
                merged.install_network.mode = mode;
            }
            if let Some(proxy_url) = network_patch.proxy_url {
                merged.install_network.proxy_url = Some(proxy_url);
            }
            if let Some(npm_registry) = network_patch.npm_registry {
                merged.install_network.npm_registry = npm_registry;
            }
        }
        if let Some(ccswitch_path) = patch.ccswitch_path {
            merged.ccswitch_path = Some(ccswitch_path);
        }
        if let Some(ccswitch_download_sources) = patch.ccswitch_download_sources {
            merged.ccswitch_download_sources = ccswitch_download_sources;
        }

        assert!(matches!(
            merged.install_network.mode,
            InstallNetworkMode::ManualProxy
        ));
        assert_eq!(
            merged.install_network.proxy_url.as_deref(),
            Some("http://127.0.0.1:7890")
        );
        assert_eq!(
            merged.ccswitch_path.as_deref(),
            Some("C:\\Program Files\\ccswitch\\ccswitch.exe")
        );
        assert_eq!(merged.ccswitch_download_sources.len(), 1);
        assert_eq!(merged.ccswitch_download_sources[0].priority, 10);
    }

    #[test]
    fn uses_prd_config_path() {
        let expected = std::path::Path::new("ai-coding-installer").join("config.json");
        let actual = std::path::Path::new("ai-coding-installer").join("config.json");
        assert_eq!(actual, expected);
    }

    #[test]
    fn recovers_malformed_config_by_backing_up_and_writing_default() {
        let dir = unique_test_dir("malformed");
        fs::create_dir_all(&dir).expect("create temp config dir");
        let path = dir.join("config.json");
        fs::write(&path, "{ definitely not json").expect("write malformed config");

        let config = get_config_from_path(&path).expect("recover malformed config");

        assert!(matches!(config.install_network.mode, InstallNetworkMode::None));
        let normalized = fs::read_to_string(&path).expect("read normalized config");
        assert!(normalized.contains("\"installNetwork\""));
        let backups = fs::read_dir(&dir)
            .expect("read temp config dir")
            .filter_map(Result::ok)
            .filter(|entry| {
                entry
                    .file_name()
                    .to_string_lossy()
                    .starts_with("config.bak.")
            })
            .collect::<Vec<_>>();
        assert_eq!(backups.len(), 1);
        let backup_name = backups[0].file_name().to_string_lossy().to_string();
        assert!(
            is_config_backup_file_name(&backup_name),
            "unexpected backup filename: {backup_name}"
        );
        let backup_content = fs::read_to_string(backups[0].path()).expect("read backup");
        assert_eq!(backup_content, "{ definitely not json");

        let _ = fs::remove_dir_all(dir);
    }

    #[test]
    fn normalizes_unknown_fields_out_of_existing_config() {
        let dir = unique_test_dir("unknown-fields");
        fs::create_dir_all(&dir).expect("create temp config dir");
        let path = dir.join("config.json");
        fs::write(
            &path,
            r#"{
  "installNetwork": {
    "mode": "none",
    "proxyUrl": null,
    "npmRegistry": "default",
    "customNpmRegistry": null,
    "pipIndexMode": "should-not-survive"
  },
  "apiKey": "should-not-survive",
  "provider": "should-not-survive",
  "pipIndexMode": "should-not-survive",
  "ccswitchPath": null,
  "ccswitchDownloadSources": []
}"#,
        )
        .expect("write config with unknown fields");

        let _config = get_config_from_path(&path).expect("read config with unknown fields");

        let normalized = fs::read_to_string(&path).expect("read normalized config");
        assert!(!normalized.contains("apiKey"));
        assert!(!normalized.contains("provider"));
        assert!(!normalized.contains("pipIndexMode"));

        let _ = fs::remove_dir_all(dir);
    }

    #[test]
    fn validates_custom_npm_registry_url_before_writing() {
        let dir = unique_test_dir("custom-registry");
        fs::create_dir_all(&dir).expect("create temp config dir");
        let path = dir.join("config.json");
        write_config_to_path(&path, &AppConfig::default()).expect("write default config");

        let error = update_config_at_path(
            &path,
            AppConfigPatch {
                install_network: Some(InstallNetworkPatch {
                    mode: None,
                    proxy_url: None,
                    npm_registry: Some(NpmRegistryOption::Custom),
                    custom_npm_registry: Some("https://user:pass@example.com/npm".to_string()),
                }),
                ccswitch_path: None,
                ccswitch_download_sources: None,
            },
        )
        .expect_err("reject registry URL with credentials");
        assert!(error.contains("custom npm registry"));

        let config = update_config_at_path(
            &path,
            AppConfigPatch {
                install_network: Some(InstallNetworkPatch {
                    mode: None,
                    proxy_url: None,
                    npm_registry: Some(NpmRegistryOption::Custom),
                    custom_npm_registry: Some("  https://registry.example.com/npm  ".to_string()),
                }),
                ccswitch_path: None,
                ccswitch_download_sources: None,
            },
        )
        .expect("accept valid custom registry URL");
        assert_eq!(
            config.install_network.custom_npm_registry.as_deref(),
            Some("https://registry.example.com/npm")
        );

        let _ = fs::remove_dir_all(dir);
    }

    #[test]
    fn custom_npm_registry_requires_valid_url_when_custom_option_is_selected() {
        let dir = unique_test_dir("custom-registry-required");
        fs::create_dir_all(&dir).expect("create temp config dir");
        let path = dir.join("config.json");
        write_config_to_path(&path, &AppConfig::default()).expect("write default config");

        let error = update_config_at_path(
            &path,
            AppConfigPatch {
                install_network: Some(InstallNetworkPatch {
                    mode: None,
                    proxy_url: None,
                    npm_registry: Some(NpmRegistryOption::Custom),
                    custom_npm_registry: Some("   ".to_string()),
                }),
                ccswitch_path: None,
                ccswitch_download_sources: None,
            },
        )
        .expect_err("reject blank custom registry URL");
        assert!(error.contains("custom npm registry"));

        let _ = fs::remove_dir_all(dir);
    }

    #[test]
    fn validates_ccswitch_download_source_urls_before_writing() {
        let dir = unique_test_dir("ccswitch-sources");
        fs::create_dir_all(&dir).expect("create temp config dir");
        let path = dir.join("config.json");
        write_config_to_path(&path, &AppConfig::default()).expect("write default config");

        let invalid_source = CcSwitchDownloadSource {
            name: "Invalid".to_string(),
            url: "ftp://example.com/ccswitch.exe".to_string(),
            priority: 1,
            enabled: true,
            kind: CcSwitchDownloadSourceKind::DirectExe,
            sha256: None,
            min_file_size_bytes: None,
        };
        let error = update_config_at_path(
            &path,
            AppConfigPatch {
                install_network: None,
                ccswitch_path: None,
                ccswitch_download_sources: Some(vec![invalid_source]),
            },
        )
        .expect_err("reject unsupported download URL scheme");
        assert!(error.contains("ccSwitch download source"));

        let valid_source = CcSwitchDownloadSource {
            name: "Valid".to_string(),
            url: "  https://example.com/ccswitch.exe  ".to_string(),
            priority: 1,
            enabled: true,
            kind: CcSwitchDownloadSourceKind::DirectExe,
            sha256: None,
            min_file_size_bytes: None,
        };
        let config = update_config_at_path(
            &path,
            AppConfigPatch {
                install_network: None,
                ccswitch_path: None,
                ccswitch_download_sources: Some(vec![valid_source]),
            },
        )
        .expect("accept valid download URL");
        assert_eq!(
            config.ccswitch_download_sources[0].url,
            "https://example.com/ccswitch.exe"
        );

        let _ = fs::remove_dir_all(dir);
    }

    #[test]
    fn rejects_url_with_path_whitespace() {
        let error = validate_http_url(
            "https://registry.example.com/npm mirror",
            "custom npm registry",
        )
        .expect_err("reject URL with whitespace outside authority");

        assert!(error.contains("whitespace"));
    }

    fn unique_test_dir(name: &str) -> PathBuf {
        std::env::temp_dir().join(format!(
            "ai-coding-config-test-{name}-{}",
            chrono::Local::now().timestamp_nanos_opt().unwrap_or_default()
        ))
    }

    fn is_config_backup_file_name(name: &str) -> bool {
        let Some(timestamp) = name
            .strip_prefix("config.bak.")
            .and_then(|value| value.strip_suffix(".json"))
        else {
            return false;
        };
        timestamp.len() == "YYYYMMDD-HHMMSS".len()
            && timestamp.as_bytes()[8] == b'-'
            && timestamp
                .chars()
                .enumerate()
                .all(|(index, character)| index == 8 || character.is_ascii_digit())
    }
}
