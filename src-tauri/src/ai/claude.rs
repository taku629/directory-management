//! Minimal Claude API client.
//!
//! Hand-rolled rather than pulling in a full SDK — we only need three
//! patterns: text chat, vision (image + text), and tool-use. Caches results
//! aggressively in the DB so we don't burn tokens on retries.

use std::path::Path;

use base64::Engine;
use reqwest::header;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

use crate::error::{AppError, AppResult};

const MODEL_DEFAULT: &str = "claude-sonnet-4-6";
const ENDPOINT: &str = "https://api.anthropic.com/v1/messages";
const VERSION: &str = "2023-06-01";

#[derive(Debug, Serialize)]
struct MessagesRequest<'a> {
    model: &'a str,
    max_tokens: u32,
    messages: Vec<MessageOut<'a>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    system: Option<&'a str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    tools: Option<Value>,
}

#[derive(Debug, Serialize)]
struct MessageOut<'a> {
    role: &'a str,
    content: Value,
}

#[derive(Debug, Deserialize)]
pub struct MessagesResponse {
    pub content: Vec<ContentBlock>,
    pub stop_reason: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(tag = "type")]
pub enum ContentBlock {
    #[serde(rename = "text")]
    Text { text: String },
    #[serde(rename = "tool_use")]
    ToolUse {
        #[allow(dead_code)]
        id: String,
        name: String,
        input: Value,
    },
}

pub struct Client {
    http: reqwest::Client,
    key: String,
    model: String,
}

impl Client {
    pub fn new(key: String, model: Option<String>) -> Self {
        Self {
            http: reqwest::Client::builder()
                .timeout(std::time::Duration::from_secs(60))
                .build()
                .expect("reqwest client"),
            key,
            model: model.unwrap_or_else(|| MODEL_DEFAULT.to_string()),
        }
    }

    /// Plain text → text.
    pub async fn text(&self, system: Option<&str>, user: &str) -> AppResult<String> {
        let req = MessagesRequest {
            model: &self.model,
            max_tokens: 1024,
            messages: vec![MessageOut {
                role: "user",
                content: json!(user),
            }],
            system,
            tools: None,
        };
        let resp = self.send(&req).await?;
        Ok(first_text(&resp))
    }

    /// Image (file path on disk) + prompt → text.
    pub async fn vision(&self, image_path: &Path, prompt: &str) -> AppResult<String> {
        let bytes = std::fs::read(image_path)?;
        let mime = mime_guess::from_path(image_path)
            .first_raw()
            .unwrap_or("image/jpeg")
            .to_string();
        if !mime.starts_with("image/") {
            return Err(AppError::Invalid("not an image".into()));
        }
        let b64 = base64::engine::general_purpose::STANDARD.encode(&bytes);

        let content = json!([
            {
                "type": "image",
                "source": {
                    "type": "base64",
                    "media_type": mime,
                    "data": b64
                }
            },
            { "type": "text", "text": prompt }
        ]);
        let req = MessagesRequest {
            model: &self.model,
            max_tokens: 512,
            messages: vec![MessageOut {
                role: "user",
                content,
            }],
            system: None,
            tools: None,
        };
        let resp = self.send(&req).await?;
        Ok(first_text(&resp))
    }

    /// Tool-use: pass a single tool schema, expect the model to fill it.
    pub async fn tool_call(
        &self,
        system: Option<&str>,
        user: &str,
        tool_name: &str,
        tool_schema: Value,
    ) -> AppResult<Value> {
        let tools = json!([{
            "name": tool_name,
            "description": "Structured response",
            "input_schema": tool_schema,
        }]);
        let req = MessagesRequest {
            model: &self.model,
            max_tokens: 1024,
            messages: vec![MessageOut {
                role: "user",
                content: json!(user),
            }],
            system,
            tools: Some(tools),
        };
        let resp = self.send(&req).await?;
        for c in resp.content {
            if let ContentBlock::ToolUse { name, input, .. } = c {
                if name == tool_name {
                    return Ok(input);
                }
            }
        }
        Err(AppError::Other(anyhow::anyhow!(
            "model did not call tool {tool_name}"
        )))
    }

    async fn send(&self, req: &MessagesRequest<'_>) -> AppResult<MessagesResponse> {
        let resp = self
            .http
            .post(ENDPOINT)
            .header("x-api-key", &self.key)
            .header("anthropic-version", VERSION)
            .header(header::CONTENT_TYPE, "application/json")
            .json(req)
            .send()
            .await
            .map_err(|e| AppError::Other(anyhow::anyhow!(e)))?;
        if !resp.status().is_success() {
            let status = resp.status();
            let body = resp.text().await.unwrap_or_default();
            return Err(AppError::Other(anyhow::anyhow!(
                "claude api {status}: {body}"
            )));
        }
        let parsed: MessagesResponse = resp
            .json()
            .await
            .map_err(|e| AppError::Other(anyhow::anyhow!(e)))?;
        Ok(parsed)
    }
}

fn first_text(resp: &MessagesResponse) -> String {
    for c in &resp.content {
        if let ContentBlock::Text { text } = c {
            return text.clone();
        }
    }
    String::new()
}
