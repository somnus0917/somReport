use std::sync::atomic::{AtomicBool, AtomicU32, Ordering};

use async_trait::async_trait;
use chrono::Utc;
use uuid::Uuid;

use crate::domain::{CaptureCapabilities, CaptureProvider, CapturedFrame};

pub struct FakeCaptureProvider {
    session_active: AtomicBool,
    frame_count: AtomicU32,
}

impl FakeCaptureProvider {
    pub fn new() -> Self {
        Self {
            session_active: AtomicBool::new(false),
            frame_count: AtomicU32::new(0),
        }
    }

    pub fn start_session(&self) {
        self.session_active.store(true, Ordering::SeqCst);
        self.frame_count.store(0, Ordering::SeqCst);
    }

    pub fn stop_session(&self) {
        self.session_active.store(false, Ordering::SeqCst);
    }

    pub fn is_session_active(&self) -> bool {
        self.session_active.load(Ordering::SeqCst)
    }

    pub fn frame_count(&self) -> u32 {
        self.frame_count.load(Ordering::SeqCst)
    }

    fn generate_test_png() -> Vec<u8> {
        // Minimal valid 1x1 white pixel PNG
        vec![
            0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A, // PNG signature
            0x00, 0x00, 0x00, 0x0D, 0x49, 0x48, 0x44, 0x52, // IHDR chunk
            0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x01, // 1x1
            0x08, 0x02, 0x00, 0x00, 0x00, 0x90, 0x77, 0x53, // 8-bit RGB
            0xDE, 0x00, 0x00, 0x00, 0x0C, 0x49, 0x44, 0x41, // IDAT chunk
            0x54, 0x08, 0xD7, 0x63, 0xF8, 0xCF, 0xC0, 0x00,
            0x00, 0x00, 0x02, 0x00, 0x01, 0xE2, 0x21, 0xBC,
            0x33, 0x00, 0x00, 0x00, 0x00, 0x49, 0x45, 0x4E, // IEND chunk
            0x44, 0xAE, 0x42, 0x60, 0x82,
        ]
    }
}

impl Default for FakeCaptureProvider {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl CaptureProvider for FakeCaptureProvider {
    async fn capabilities(&self) -> CaptureCapabilities {
        CaptureCapabilities {
            can_capture_primary: true,
            can_capture_secondary: false,
            supports_multi_monitor: false,
            max_resolution: (1920, 1080),
        }
    }

    async fn capture(&self) -> Result<CapturedFrame, String> {
        if !self.session_active.load(Ordering::SeqCst) {
            return Err("No active capture session".to_string());
        }
        self.frame_count.fetch_add(1, Ordering::SeqCst);
        Ok(CapturedFrame {
            id: Uuid::new_v4().to_string(),
            captured_at: Utc::now(),
            png_data: Self::generate_test_png(),
            width: 1,
            height: 1,
            display_index: 0,
            image_hash: None,
        })
    }

    async fn capture_all_displays(&self) -> Result<Vec<CapturedFrame>, String> {
        let frame = self.capture().await?;
        Ok(vec![frame])
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_fake_provider_lifecycle() {
        let provider = FakeCaptureProvider::new();
        assert!(!provider.is_session_active());

        provider.start_session();
        assert!(provider.is_session_active());
        assert_eq!(provider.frame_count(), 0);

        let frame = provider.capture().await.unwrap();
        assert_eq!(frame.width, 1);
        assert_eq!(frame.height, 1);
        assert!(!frame.png_data.is_empty());
        assert_eq!(provider.frame_count(), 1);

        let frames = provider.capture_all_displays().await.unwrap();
        assert_eq!(frames.len(), 1);
        assert_eq!(provider.frame_count(), 2);

        provider.stop_session();
        assert!(!provider.is_session_active());
    }

    #[tokio::test]
    async fn test_fake_provider_capture_without_session_fails() {
        let provider = FakeCaptureProvider::new();
        let result = provider.capture().await;
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "No active capture session");
    }
}
