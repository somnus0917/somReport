use async_trait::async_trait;
use serde::{Deserialize, Serialize};

use super::activity::Category;
use super::capture::CapturedFrame;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VisionItem {
    pub category: Category,
    pub summary: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub detail: Option<String>,
    pub confidence: f64,
    pub is_work_related: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VisionResult {
    pub items: Vec<VisionItem>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct TokenUsage {
    pub input_tokens: i64,
    pub output_tokens: i64,
}

#[derive(Debug, Clone)]
pub struct ProviderResponse<T> {
    pub value: T,
    pub usage: TokenUsage,
}

#[async_trait]
pub trait VisionProvider: Send + Sync {
    async fn analyze(
        &self,
        frame: &CapturedFrame,
    ) -> Result<ProviderResponse<VisionResult>, String>;
}

#[async_trait]
pub trait TextProvider: Send + Sync {
    async fn generate(&self, prompt: &str) -> Result<ProviderResponse<String>, String>;
}
