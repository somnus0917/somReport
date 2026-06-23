pub mod anthropic;
pub mod openai;
pub mod validation;

use std::sync::Arc;

use crate::domain::{ProviderConfig, TextProvider, VisionProvider};

pub fn resolve_api_key(config: &ProviderConfig) -> Result<String, String> {
    if let Some(key) = config
        .api_key
        .as_deref()
        .filter(|key| !key.trim().is_empty())
    {
        return Ok(key.to_string());
    }

    if let Ok(entry) = keyring::Entry::new("daytrace", &config.name) {
        if let Ok(key) = entry.get_password() {
            if !key.trim().is_empty() {
                return Ok(key);
            }
        }
    }

    if let Some(variable) = &config.api_key_env_var {
        if let Ok(key) = std::env::var(variable) {
            if !key.trim().is_empty() {
                return Ok(key);
            }
        }
    }

    Err(format!(
        "No API key configured for {}. Save one in Settings or set {}.",
        config.name,
        config
            .api_key_env_var
            .as_deref()
            .unwrap_or("an API-key environment variable")
    ))
}

pub fn create_vision_provider(config: &ProviderConfig) -> Result<Arc<dyn VisionProvider>, String> {
    let key = resolve_api_key(config)?;
    match config.name.as_str() {
        "openai" | "qwen" => Ok(Arc::new(
            openai::OpenAIProvider::new(&key)
                .with_base_url(&config.api_url)
                .with_vision_model(&config.model),
        )),
        "anthropic" => Ok(Arc::new(
            anthropic::AnthropicProvider::new(&key)
                .with_base_url(&config.api_url)
                .with_vision_model(&config.model),
        )),
        other => Err(format!("Unsupported vision provider: {other}")),
    }
}

pub fn create_text_provider(config: &ProviderConfig) -> Result<Arc<dyn TextProvider>, String> {
    let key = resolve_api_key(config)?;
    match config.name.as_str() {
        "openai" | "qwen" => Ok(Arc::new(
            openai::OpenAIProvider::new(&key)
                .with_base_url(&config.api_url)
                .with_text_model(&config.model),
        )),
        "anthropic" => Ok(Arc::new(
            anthropic::AnthropicProvider::new(&key)
                .with_base_url(&config.api_url)
                .with_text_model(&config.model),
        )),
        other => Err(format!("Unsupported text provider: {other}")),
    }
}
