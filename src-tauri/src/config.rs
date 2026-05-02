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

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AppConfig {
    pub install_network: InstallNetworkConfig,
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
        }
    }
}

pub fn get_config() -> Result<AppConfig, String> {
    let path = config_path()?;
    if !path.exists() {
        let config = AppConfig::default();
        write_config(&config)?;
        return Ok(config);
    }

    let content = fs::read_to_string(&path).map_err(|error| error.to_string())?;
    serde_json::from_str::<AppConfig>(&content).map_err(|error| error.to_string())
}

pub fn update_config(patch: AppConfigPatch) -> Result<AppConfig, String> {
    let mut config = get_config()?;
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
    write_config(&config)?;
    Ok(config)
}

pub fn reset_config() -> Result<AppConfig, String> {
    let config = AppConfig::default();
    write_config(&config)?;
    Ok(config)
}

fn write_config(config: &AppConfig) -> Result<(), String> {
    let path = config_path()?;
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_config_omits_unused_pip_index_mode() {
        let json = serde_json::to_value(AppConfig::default()).expect("serialize config");
        assert!(json.get("pipIndexMode").is_none());
        assert_eq!(json["installNetwork"]["mode"], "none");
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

        assert!(matches!(
            merged.install_network.mode,
            InstallNetworkMode::ManualProxy
        ));
        assert_eq!(
            merged.install_network.proxy_url.as_deref(),
            Some("http://127.0.0.1:7890")
        );
    }

    #[test]
    fn uses_prd_config_path() {
        let expected = std::path::Path::new("ai-coding-installer").join("config.json");
        let actual = std::path::Path::new("ai-coding-installer").join("config.json");
        assert_eq!(actual, expected);
    }
}
