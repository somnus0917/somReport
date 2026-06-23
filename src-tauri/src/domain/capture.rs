use async_trait::async_trait;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CaptureCapabilities {
    pub can_capture_primary: bool,
    pub can_capture_secondary: bool,
    pub supports_multi_monitor: bool,
    pub max_resolution: (u32, u32),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CapturedFrame {
    pub id: String,
    pub captured_at: DateTime<Utc>,
    /// Encoded image bytes. The historical field name is retained for storage/API compatibility.
    pub png_data: Vec<u8>,
    pub mime_type: String,
    pub width: u32,
    pub height: u32,
    pub display_index: u32,
    pub image_hash: Option<String>,
}

#[async_trait]
pub trait CaptureProvider: Send + Sync {
    async fn capabilities(&self) -> CaptureCapabilities;
    async fn capture(&self) -> Result<CapturedFrame, String>;
    async fn capture_all_displays(&self) -> Result<Vec<CapturedFrame>, String>;
}
