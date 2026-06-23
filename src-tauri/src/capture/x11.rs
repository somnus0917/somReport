use async_trait::async_trait;
use chrono::Utc;
use image::{codecs::jpeg::JpegEncoder, DynamicImage};
use uuid::Uuid;
use xcap::Monitor;

use crate::domain::capture::{CaptureCapabilities, CaptureProvider, CapturedFrame};

const MAX_EDGE: u32 = 1600;
const JPEG_QUALITY: u8 = 75;

pub struct X11CaptureProvider;

impl X11CaptureProvider {
    pub fn new() -> Self {
        Self
    }

    fn encode_jpeg(
        img: &DynamicImage,
        max_edge: u32,
        quality: u8,
    ) -> Result<(Vec<u8>, u32, u32), String> {
        let (w, h) = (img.width(), img.height());
        let resized;
        let target = if w > max_edge || h > max_edge {
            resized = img.resize(max_edge, max_edge, image::imageops::FilterType::Lanczos3);
            &resized
        } else {
            img
        };

        let mut buf = std::io::Cursor::new(Vec::new());
        let encoder = JpegEncoder::new_with_quality(&mut buf, quality);
        target
            .write_with_encoder(encoder)
            .map_err(|e| e.to_string())?;
        Ok((buf.into_inner(), target.width(), target.height()))
    }
}

impl Default for X11CaptureProvider {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl CaptureProvider for X11CaptureProvider {
    async fn capabilities(&self) -> CaptureCapabilities {
        CaptureCapabilities {
            can_capture_primary: true,
            can_capture_secondary: true,
            supports_multi_monitor: true,
            max_resolution: (MAX_EDGE, MAX_EDGE),
        }
    }

    async fn capture(&self) -> Result<CapturedFrame, String> {
        let monitors = Monitor::all().map_err(|e| format!("Failed to enumerate monitors: {e}"))?;
        let monitor = monitors.into_iter().next().ok_or("No monitors found")?;

        let img = monitor
            .capture_image()
            .map_err(|e| format!("Capture failed: {e}"))?;
        let dynamic = DynamicImage::ImageRgba8(img);
        let (data, width, height) = Self::encode_jpeg(&dynamic, MAX_EDGE, JPEG_QUALITY)?;

        Ok(CapturedFrame {
            id: Uuid::new_v4().to_string(),
            captured_at: Utc::now(),
            png_data: data,
            mime_type: "image/jpeg".to_string(),
            width,
            height,
            display_index: 0,
            image_hash: None,
        })
    }

    async fn capture_all_displays(&self) -> Result<Vec<CapturedFrame>, String> {
        let monitors = Monitor::all().map_err(|e| format!("Failed to enumerate monitors: {e}"))?;
        let mut frames = Vec::with_capacity(monitors.len());

        for (index, monitor) in monitors.into_iter().enumerate() {
            let img = monitor
                .capture_image()
                .map_err(|e| format!("Capture failed for display {index}: {e}"))?;
            let dynamic = DynamicImage::ImageRgba8(img);
            let (data, width, height) = Self::encode_jpeg(&dynamic, MAX_EDGE, JPEG_QUALITY)?;

            frames.push(CapturedFrame {
                id: Uuid::new_v4().to_string(),
                captured_at: Utc::now(),
                png_data: data,
                mime_type: "image/jpeg".to_string(),
                width,
                height,
                display_index: index as u32,
                image_hash: None,
            });
        }

        Ok(frames)
    }
}
