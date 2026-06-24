use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProviderConfig {
    pub name: String,
    pub api_key_env_var: Option<String>,
    pub api_url: String,
    pub model: String,
    #[serde(default)]
    pub input_cost_per_million_cents: f64,
    #[serde(default)]
    pub output_cost_per_million_cents: f64,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ModelConnectionStatus {
    pub success: Option<bool>,
    pub tested_at: Option<String>,
    pub message: Option<String>,
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
    #[serde(default = "default_data_retention_days")]
    pub data_retention_days: u32,
    #[serde(default)]
    pub vision_connection: ModelConnectionStatus,
    #[serde(default)]
    pub text_connection: ModelConnectionStatus,
}

impl Default for AppSettings {
    fn default() -> Self {
        Self {
            vision_provider: ProviderConfig {
                name: "qwen".to_string(),
                api_key_env_var: Some("QWEN_API_KEY".to_string()),
                api_url: "https://dashscope.aliyuncs.com/compatible-mode/v1".to_string(),
                model: "qwen-vl-max".to_string(),
                input_cost_per_million_cents: 0.0,
                output_cost_per_million_cents: 0.0,
            },
            text_provider: ProviderConfig {
                name: "qwen".to_string(),
                api_key_env_var: Some("QWEN_API_KEY".to_string()),
                api_url: "https://dashscope.aliyuncs.com/compatible-mode/v1".to_string(),
                model: "qwen-plus".to_string(),
                input_cost_per_million_cents: 0.0,
                output_cost_per_million_cents: 0.0,
            },
            capture_interval_secs: 30,
            idle_timeout_secs: 300,
            max_daily_cost_cents: 500,
            auto_start: false,
            notify_on_report: true,
            data_retention_days: default_data_retention_days(),
            vision_connection: ModelConnectionStatus::default(),
            text_connection: ModelConnectionStatus::default(),
        }
    }
}

fn default_data_retention_days() -> u32 {
    30
}
