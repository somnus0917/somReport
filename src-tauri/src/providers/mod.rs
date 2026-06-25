pub mod anthropic;
pub mod openai;
pub mod prompts;
pub mod validation;

use std::sync::Arc;

use crate::domain::{ProviderConfig, TextProvider, VisionProvider};

pub fn resolve_api_key(config: &ProviderConfig) -> Result<String, String> {
    // Credentials are intentionally supplied only through the process environment.
    if let Some(variable) = &config.api_key_env_var {
        log::info!("Trying env var {} for {}", variable, config.name);
        if let Ok(key) = std::env::var(variable) {
            if !key.trim().is_empty() {
                log::info!("Found key in env var {} for {}", variable, config.name);
                return Ok(key);
            }
        }
    }

    Err(format!(
        "No API key configured for {}. Set {} before launching the app.",
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
