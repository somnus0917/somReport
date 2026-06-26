pub mod anthropic;
pub mod openai;
pub mod prompts;
pub mod validation;

use aes_gcm::{
    aead::{Aead, KeyInit},
    Aes256Gcm, Nonce,
};
use base64::Engine;
use std::sync::Arc;

use crate::domain::{ProviderConfig, TextProvider, VisionProvider};

static ENCRYPTION_KEY: std::sync::OnceLock<[u8; 32]> = std::sync::OnceLock::new();

fn get_encryption_key() -> &'static [u8; 32] {
    ENCRYPTION_KEY.get_or_init(|| {
        let key_path = crate::platform::paths::app_data_dir().join("enc.key");
        if let Ok(key_bytes) = std::fs::read(&key_path) {
            if key_bytes.len() == 32 {
                let mut key = [0u8; 32];
                key.copy_from_slice(&key_bytes);
                return key;
            }
        }

        let mut key = [0u8; 32];
        let uuid1 = uuid::Uuid::new_v4();
        let uuid2 = uuid::Uuid::new_v4();
        key[0..16].copy_from_slice(uuid1.as_bytes());
        key[16..32].copy_from_slice(uuid2.as_bytes());
        let _ = std::fs::write(&key_path, &key);
        key
    })
}

pub fn encrypt_key(plaintext: &str) -> Result<String, String> {
    let key_bytes = get_encryption_key();
    let key = aes_gcm::Key::<Aes256Gcm>::from_slice(key_bytes);
    let cipher = Aes256Gcm::new(key);

    // Nonce must be 96 bits (12 bytes)
    let uuid_nonce = uuid::Uuid::new_v4();
    let nonce_bytes = &uuid_nonce.as_bytes()[0..12];
    let nonce = Nonce::from_slice(nonce_bytes);

    let ciphertext = cipher
        .encrypt(nonce, plaintext.as_bytes())
        .map_err(|e| format!("Encryption error: {:?}", e))?;

    let mut combined = Vec::with_capacity(12 + ciphertext.len());
    combined.extend_from_slice(nonce_bytes);
    combined.extend_from_slice(&ciphertext);

    Ok(format!(
        "enc:{}",
        base64::prelude::BASE64_STANDARD.encode(combined)
    ))
}

pub fn decrypt_key(encoded: &str) -> Result<String, String> {
    if !encoded.starts_with("enc:") {
        return Err("Not an encrypted key".to_string());
    }

    let ciphertext_b64 = &encoded[4..];
    let combined = base64::prelude::BASE64_STANDARD
        .decode(ciphertext_b64)
        .map_err(|e| format!("Base64 decode error: {:?}", e))?;

    if combined.len() < 12 {
        return Err("Ciphertext too short".to_string());
    }

    let key_bytes = get_encryption_key();
    let key = aes_gcm::Key::<Aes256Gcm>::from_slice(key_bytes);
    let cipher = Aes256Gcm::new(key);

    let nonce_bytes = &combined[0..12];
    let ciphertext = &combined[12..];
    let nonce = Nonce::from_slice(nonce_bytes);

    let decrypted = cipher
        .decrypt(nonce, ciphertext)
        .map_err(|e| format!("Decryption error: {:?}", e))?;

    String::from_utf8(decrypted).map_err(|e| format!("Invalid UTF-8: {:?}", e))
}

pub fn resolve_api_key(config: &ProviderConfig, _role: &str) -> Result<String, String> {
    // 1. If an actual (non-placeholder, non-empty) key is provided inline, use it
    if let Some(key) = &config.api_key {
        if !key.trim().is_empty() && key != "******" {
            if key.starts_with("enc:") {
                // Decrypt the saved key
                if let Ok(decrypted) = decrypt_key(key) {
                    return Ok(decrypted);
                }
            } else {
                // Use the raw key directly
                return Ok(key.clone());
            }
        }
    }

    // 2. Try to read from process environment variables
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
        "No API key configured for {}. Please enter an API key or set {} before launching the app.",
        config.name,
        config
            .api_key_env_var
            .as_deref()
            .unwrap_or("an API-key environment variable")
    ))
}

pub fn create_vision_provider(config: &ProviderConfig) -> Result<Arc<dyn VisionProvider>, String> {
    let key = resolve_api_key(config, "vision")?;
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
    let key = resolve_api_key(config, "text")?;
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_encryption_decryption() {
        let plaintext = "sk-proj-my-secret-key-12345";
        let encrypted = encrypt_key(plaintext).expect("failed to encrypt");
        assert!(encrypted.starts_with("enc:"));

        let decrypted = decrypt_key(&encrypted).expect("failed to decrypt");
        assert_eq!(plaintext, decrypted);
    }
}
