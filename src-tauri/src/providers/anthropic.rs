use async_trait::async_trait;
use base64::Engine;
use serde::{Deserialize, Serialize};

use crate::domain::capture::CapturedFrame;
use crate::domain::provider::{
    ProviderResponse, TextProvider, TokenUsage, VisionProvider, VisionResult,
};

use super::validation::validate_vision_result;

const VISION_SYSTEM_PROMPT: &str = r#"You are an activity classifier for a desktop time-tracker.
Analyze the screenshot and return a JSON object with this exact shape:
{
  "items": [
    {
      "category": "development|meeting|communication|documentation|research|design|other",
      "summary": "≤80 chars describing what the user is doing",
      "detail": "optional ≤240 chars with extra context",
      "confidence": 0.0–1.0,
      "is_work_related": true|false
    }
  ]
}
Guidelines:
- Use multiple items if the screen shows distinct activities.
- Prefer specific categories (development, meeting) over generic ones (other).
- Confidence should reflect how clearly the screenshot shows the activity.
- "other" is the fallback; use it only when nothing else fits.
- Return ONLY valid JSON, no markdown fences, no explanation."#;

const TEXT_SYSTEM_PROMPT: &str = r#"You are a report generator for a work time-tracker.
Given a list of activities with timestamps and categories, produce a concise daily summary in Markdown format.
Structure:
- One section per category with activities listed.
- Each activity: bullet with time range, summary, and any detail.
- At the end, a brief "Highlights" section with 2-3 key takeaways.
Keep it professional and factual. Return ONLY the Markdown report."#;

#[derive(Debug, Serialize)]
struct ImageSource<'a> {
    #[serde(rename = "type")]
    source_type: &'a str,
    media_type: &'a str,
    data: &'a str,
}

#[derive(Debug, Serialize)]
struct ContentBlock<'a> {
    #[serde(rename = "type")]
    block_type: &'a str,
    #[serde(skip_serializing_if = "Option::is_none")]
    text: Option<&'a str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    source: Option<ImageSource<'a>>,
}

#[derive(Debug, Serialize)]
struct Message<'a> {
    role: &'a str,
    content: Vec<ContentBlock<'a>>,
}

#[derive(Debug, Serialize)]
struct MessagesRequest<'a> {
    model: &'a str,
    max_tokens: u32,
    #[serde(skip_serializing_if = "Option::is_none")]
    system: Option<&'a str>,
    messages: Vec<Message<'a>>,
}

#[derive(Debug, Deserialize)]
struct MessagesResponse {
    content: Vec<ResponseContentBlock>,
    usage: Option<MessageUsage>,
}

#[derive(Debug, Deserialize)]
struct MessageUsage {
    input_tokens: Option<i64>,
    output_tokens: Option<i64>,
}

#[derive(Debug, Deserialize)]
struct ResponseContentBlock {
    #[serde(rename = "type")]
    block_type: Option<String>,
    text: Option<String>,
}

pub struct AnthropicProvider {
    client: reqwest::Client,
    api_key: String,
    base_url: String,
    vision_model: String,
    text_model: String,
}

impl AnthropicProvider {
    pub fn new(api_key: &str) -> Self {
        Self {
            client: reqwest::Client::new(),
            api_key: api_key.to_string(),
            base_url: "https://api.anthropic.com".to_string(),
            vision_model: "claude-sonnet-4-20250514".to_string(),
            text_model: "claude-sonnet-4-20250514".to_string(),
        }
    }

    pub fn with_base_url(mut self, base_url: &str) -> Self {
        self.base_url = base_url.trim_end_matches('/').to_string();
        self
    }

    pub fn with_vision_model(mut self, model: &str) -> Self {
        self.vision_model = model.to_string();
        self
    }

    pub fn with_text_model(mut self, model: &str) -> Self {
        self.text_model = model.to_string();
        self
    }

    async fn send_message(
        &self,
        model: &str,
        system: Option<&str>,
        messages: Vec<Message<'_>>,
        max_tokens: u32,
    ) -> Result<ProviderResponse<String>, String> {
        let url = format!("{}/v1/messages", self.base_url);

        let body = MessagesRequest {
            model,
            max_tokens,
            system,
            messages,
        };

        let resp = self
            .client
            .post(&url)
            .header("x-api-key", &self.api_key)
            .header("anthropic-version", "2023-06-01")
            .header("Content-Type", "application/json")
            .json(&body)
            .send()
            .await
            .map_err(|e| format!("Request failed: {e}"))?;

        if !resp.status().is_success() {
            let status = resp.status();
            let text = resp.text().await.unwrap_or_default();
            return Err(format!("API error {status}: {text}"));
        }

        let msg: MessagesResponse = resp
            .json()
            .await
            .map_err(|e| format!("Failed to parse response: {e}"))?;

        let content = msg
            .content
            .iter()
            .find(|b| b.block_type.as_deref() == Some("text"))
            .and_then(|b| b.text.clone())
            .ok_or_else(|| "No text content in response".to_string())?;
        let usage = msg.usage.unwrap_or(MessageUsage {
            input_tokens: None,
            output_tokens: None,
        });
        Ok(ProviderResponse {
            value: content,
            usage: TokenUsage {
                input_tokens: usage.input_tokens.unwrap_or_default(),
                output_tokens: usage.output_tokens.unwrap_or_default(),
            },
        })
    }
}

#[async_trait]
impl VisionProvider for AnthropicProvider {
    async fn analyze(
        &self,
        frame: &CapturedFrame,
    ) -> Result<ProviderResponse<VisionResult>, String> {
        let b64 = base64::engine::general_purpose::STANDARD.encode(&frame.png_data);

        let messages = vec![Message {
            role: "user",
            content: vec![
                ContentBlock {
                    block_type: "text",
                    text: Some("What is the user doing in this screenshot?"),
                    source: None,
                },
                ContentBlock {
                    block_type: "image",
                    text: None,
                    source: Some(ImageSource {
                        source_type: "base64",
                        media_type: &frame.mime_type,
                        data: &b64,
                    }),
                },
            ],
        }];

        let response = self
            .send_message(
                &self.vision_model,
                Some(VISION_SYSTEM_PROMPT),
                messages,
                1024,
            )
            .await?;
        Ok(ProviderResponse {
            value: validate_vision_result(&response.value)?,
            usage: response.usage,
        })
    }
}

#[async_trait]
impl TextProvider for AnthropicProvider {
    async fn generate(&self, prompt: &str) -> Result<ProviderResponse<String>, String> {
        let messages = vec![Message {
            role: "user",
            content: vec![ContentBlock {
                block_type: "text",
                text: Some(prompt),
                source: None,
            }],
        }];

        self.send_message(&self.text_model, Some(TEXT_SYSTEM_PROMPT), messages, 4096)
            .await
    }
}
