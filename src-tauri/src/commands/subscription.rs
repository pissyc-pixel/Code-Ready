use crate::config::{self, AppConfig};

#[tauri::command]
pub async fn open_subscription_page() -> Result<(), String> {
    let config = config::get_config()?;
    let url = configured_subscription_page_url(&config)?;

    tauri_plugin_opener::open_url(url.as_str(), None::<&str>)
        .map_err(|error| format!("failed to open subscription page: {error}"))?;

    Ok(())
}

fn configured_subscription_page_url(config: &AppConfig) -> Result<String, String> {
    config
        .subscription_page_url
        .as_deref()
        .map(str::trim)
        .filter(|url| !url.is_empty())
        .map(ToString::to_string)
        .ok_or_else(|| "尚未配置节点订阅网页地址".to_string())
}

#[cfg(test)]
mod tests {
    use super::configured_subscription_page_url;
    use crate::config::AppConfig;

    #[test]
    fn reports_clear_error_when_subscription_page_url_is_missing() {
        let config = AppConfig::default();

        let error = configured_subscription_page_url(&config)
            .expect_err("missing subscription page URL should fail");

        assert!(error.contains("尚未配置节点订阅网页地址"));
    }

    #[test]
    fn returns_configured_subscription_page_url_without_reading_content() {
        let mut config = AppConfig::default();
        config.subscription_page_url = Some("https://subscriptions.example.com".to_string());

        let url = configured_subscription_page_url(&config)
            .expect("configured subscription page URL should be returned");

        assert_eq!(url, "https://subscriptions.example.com");
    }
}
