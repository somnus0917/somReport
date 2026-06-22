use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProviderConfig {
    pub name: String,
    pub api_key_env_var: Option<String>,
    pub api_key: Option<String>,
    pub api_url: String,
    pub model: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppSettings {
    pub vision_provider: ProviderConfig,
    pub text_provider: ProviderConfig,
    pub capture_interval_secs: u32,
    pub idle_timeout_secs: u32,
    pub max_daily_cost_cents: u32,
    pub auto_start: bool,
    pub notify_on_report: bool,
}

impl Default for AppSettings {
    fn default() -> Self {
        Self {
            vision_provider: ProviderConfig {
                name: "openai".to_string(),
                api_key_env_var: Some("OPENAI_API_KEY".to_string()),
                api_key: None,
                api_url: "https://api.openai.com/v1".to_string(),
                model: "gpt-4o-mini".to_string(),
            },
            text_provider: ProviderConfig {
                name: "openai".to_string(),
                api_key_env_var: Some("OPENAI_API_KEY".to_string()),
                api_key: None,
                api_url: "https://api.openai.com/v1".to_string(),
                model: "gpt-4o-mini".to_string(),
            },
            capture_interval_secs: 30,
            idle_timeout_secs: 300,
            max_daily_cost_cents: 500,
            auto_start: false,
            notify_on_report: true,
        }
    }
}
