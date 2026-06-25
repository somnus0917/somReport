use async_trait::async_trait;
use base64::Engine;
use serde::{Deserialize, Serialize};

use crate::domain::capture::CapturedFrame;
use crate::domain::provider::{
    ProviderResponse, TextProvider, TokenUsage, VisionProvider, VisionResult,
};

use super::validation::validate_vision_result;
use super::prompts::{VISION_SYSTEM_PROMPT, TEXT_SYSTEM_PROMPT};

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
    usage: Option<ChatUsage>,
}

#[derive(Debug, Deserialize)]
struct ChatUsage {
    prompt_tokens: Option<i64>,
    completion_tokens: Option<i64>,
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
    ) -> Result<ProviderResponse<String>, String> {
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

        let content = chat
            .choices
            .first()
            .and_then(|c| c.message.content.clone())
            .ok_or_else(|| "No content in response".to_string())?;
        let usage = chat.usage.unwrap_or(ChatUsage {
            prompt_tokens: None,
            completion_tokens: None,
        });
        Ok(ProviderResponse {
            value: content,
            usage: TokenUsage {
                input_tokens: usage.prompt_tokens.unwrap_or_default(),
                output_tokens: usage.completion_tokens.unwrap_or_default(),
            },
        })
    }
}

#[async_trait]
impl VisionProvider for OpenAIProvider {
    async fn analyze(
        &self,
        frame: &CapturedFrame,
    ) -> Result<ProviderResponse<VisionResult>, String> {
        let b64 = base64::engine::general_purpose::STANDARD.encode(&frame.image_data);
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

        let response = self.send_chat(&self.vision_model, messages, 1024).await?;
        Ok(ProviderResponse {
            value: validate_vision_result(&response.value)?,
            usage: response.usage,
        })
    }
}

#[async_trait]
impl TextProvider for OpenAIProvider {
    async fn generate(&self, prompt: &str) -> Result<ProviderResponse<String>, String> {
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
