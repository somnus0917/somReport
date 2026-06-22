use serde::Deserialize;

use crate::domain::activity::Category;
use crate::domain::provider::{VisionItem, VisionResult};

#[derive(Deserialize)]
struct RawVisionItem {
    category: String,
    summary: String,
    detail: Option<String>,
    confidence: f64,
    is_work_related: bool,
}

#[derive(Deserialize)]
struct RawVisionResult {
    items: Vec<RawVisionItem>,
}

fn truncate_to_char_boundary(s: &str, max_chars: usize) -> String {
    if s.chars().count() <= max_chars {
        s.to_string()
    } else {
        s.chars().take(max_chars).collect()
    }
}

pub fn validate_vision_result(raw_json: &str) -> Result<VisionResult, String> {
    let raw: RawVisionResult =
        serde_json::from_str(raw_json).map_err(|e| format!("Invalid JSON: {e}"))?;

    if raw.items.is_empty() {
        return Err("items array is empty".to_string());
    }

    let items: Vec<VisionItem> = raw
        .items
        .into_iter()
        .map(|ri| VisionItem {
            category: Category::from_str(&ri.category.to_lowercase()),
            summary: truncate_to_char_boundary(&ri.summary, 80),
            detail: ri.detail.map(|d| truncate_to_char_boundary(&d, 240)),
            confidence: ri.confidence.clamp(0.0, 1.0),
            is_work_related: ri.is_work_related,
        })
        .collect();

    Ok(VisionResult { items })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn valid_json() {
        let json = r#"{
            "items": [{
                "category": "development",
                "summary": "Coding in Rust",
                "detail": "Writing tests",
                "confidence": 0.95,
                "is_work_related": true
            }]
        }"#;
        let result = validate_vision_result(json);
        assert!(result.is_ok());
        let vr = result.unwrap();
        assert_eq!(vr.items.len(), 1);
        assert_eq!(vr.items[0].category, Category::Development);
        assert_eq!(vr.items[0].summary, "Coding in Rust");
        assert_eq!(vr.items[0].detail, Some("Writing tests".to_string()));
        assert_eq!(vr.items[0].confidence, 0.95);
        assert!(vr.items[0].is_work_related);
    }

    #[test]
    fn invalid_json_fails() {
        let result = validate_vision_result("{not valid json");
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Invalid JSON"));
    }

    #[test]
    fn empty_items_fails() {
        let json = r#"{ "items": [] }"#;
        let result = validate_vision_result(json);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "items array is empty");
    }

    #[test]
    fn truncates_long_summary() {
        let long_summary = "A".repeat(100);
        let json = format!(
            r#"{{
            "items": [{{
                "category": "development",
                "summary": "{long_summary}",
                "confidence": 0.9,
                "is_work_related": true
            }}]
        }}"#
        );
        let result = validate_vision_result(&json).unwrap();
        assert_eq!(result.items[0].summary.len(), 80);
    }

    #[test]
    fn clamps_confidence() {
        let json = r#"{
            "items": [
                {
                    "category": "meeting",
                    "summary": "standup",
                    "confidence": 1.5,
                    "is_work_related": true
                },
                {
                    "category": "meeting",
                    "summary": "retro",
                    "confidence": -0.3,
                    "is_work_related": true
                }
            ]
        }"#;
        let result = validate_vision_result(json).unwrap();
        assert_eq!(result.items[0].confidence, 1.0);
        assert_eq!(result.items[1].confidence, 0.0);
    }

    #[test]
    fn unknown_category_becomes_other() {
        let json = r#"{
            "items": [{
                "category": "lunchbreak",
                "summary": "eating",
                "confidence": 0.8,
                "is_work_related": false
            }]
        }"#;
        let result = validate_vision_result(json).unwrap();
        assert_eq!(result.items[0].category, Category::Other);
    }
}
