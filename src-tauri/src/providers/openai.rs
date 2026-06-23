use async_trait::async_trait;
use base64::Engine;
use serde::{Deserialize, Serialize};

use crate::domain::capture::CapturedFrame;
use crate::domain::provider::{TextProvider, VisionProvider, VisionResult};

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
struct ChatMessage<'a> {
    role: &'a str,
    content: MessageContent<'a>,
}

#[derive(Debug, Serialize)]
#[serde(untagged)]
enum MessageContent<'a> {
    Text(&'a str),
    Parts(Vec<ContentPart<'a>>),
}

#[derive(Debug, Serialize)]
struct ContentPart<'a> {
    #[serde(rename = "type")]
    part_type: &'a str,
    #[serde(skip_serializing_if = "Option::is_none")]
    text: Option<&'a str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    image_url: Option<ImageUrl<'a>>,
}

#[derive(Debug, Serialize)]
struct ImageUrl<'a> {
    url: &'a str,
    detail: &'a str,
}

#[derive(Debug, Serialize)]
struct ChatRequest<'a> {
    model: &'a str,
    messages: Vec<ChatMessage<'a>>,
    max_tokens: u32,
    temperature: f64,
}

#[derive(Debug, Deserialize)]
struct ChatResponse {
    choices: Vec<ChatChoice>,
}

#[derive(Debug, Deserialize)]
struct ChatChoice {
    message: ChatResponseMessage,
}

#[derive(Debug, Deserialize)]
struct ChatResponseMessage {
    content: Option<String>,
}

pub struct OpenAIProvider {
    client: reqwest::Client,
    api_key: String,
    base_url: String,
    vision_model: String,
    text_model: String,
}

impl OpenAIProvider {
    pub fn new(api_key: &str) -> Self {
        Self {
            client: reqwest::Client::new(),
            api_key: api_key.to_string(),
            base_url: "https://api.openai.com/v1".to_string(),
            vision_model: "gpt-4o".to_string(),
            text_model: "gpt-4o".to_string(),
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

    async fn send_chat(
        &self,
        model: &str,
        messages: Vec<ChatMessage<'_>>,
        max_tokens: u32,
    ) -> Result<String, String> {
        let url = format!("{}/chat/completions", self.base_url);

        let body = ChatRequest {
            model,
            messages,
            max_tokens,
            temperature: 0.2,
        };

        let resp = self
            .client
            .post(&url)
            .header("Authorization", format!("Bearer {}", self.api_key))
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

        let chat: ChatResponse = resp
            .json()
            .await
            .map_err(|e| format!("Failed to parse response: {e}"))?;

        chat.choices
            .first()
            .and_then(|c| c.message.content.clone())
            .ok_or_else(|| "No content in response".to_string())
    }
}

#[async_trait]
impl VisionProvider for OpenAIProvider {
    async fn analyze(&self, frame: &CapturedFrame) -> Result<VisionResult, String> {
        let b64 = base64::engine::general_purpose::STANDARD.encode(&frame.png_data);
        let data_url = format!("data:{};base64,{b64}", frame.mime_type);

        let messages = vec![
            ChatMessage {
                role: "system",
                content: MessageContent::Text(VISION_SYSTEM_PROMPT),
            },
            ChatMessage {
                role: "user",
                content: MessageContent::Parts(vec![
                    ContentPart {
                        part_type: "text",
                        text: Some("What is the user doing in this screenshot?"),
                        image_url: None,
                    },
                    ContentPart {
                        part_type: "image_url",
                        text: None,
                        image_url: Some(ImageUrl {
                            url: &data_url,
                            detail: "high",
                        }),
                    },
                ]),
            },
        ];

        let content = self.send_chat(&self.vision_model, messages, 1024).await?;
        validate_vision_result(&content)
    }
}

#[async_trait]
impl TextProvider for OpenAIProvider {
    async fn generate(&self, prompt: &str) -> Result<String, String> {
        let messages = vec![
            ChatMessage {
                role: "system",
                content: MessageContent::Text(TEXT_SYSTEM_PROMPT),
            },
            ChatMessage {
                role: "user",
                content: MessageContent::Text(prompt),
            },
        ];

        self.send_chat(&self.text_model, messages, 4096).await
    }
}
