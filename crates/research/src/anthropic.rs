//! Anthropic API provider for research relevance scoring.
//!
//! Direct Anthropic API calls - no intake-agent dependency.

use anyhow::{Context, Result};
use serde::Deserialize;
use std::fmt;

/// API key error.
#[derive(Debug)]
pub struct ApiKeyError;

impl std::error::Error for ApiKeyError {}

impl fmt::Display for ApiKeyError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "ANTHROPIC_API_KEY not set")
    }
}

/// Messages for AI API.
#[derive(Debug, Clone)]
pub struct Message {
    pub role: Role,
    pub content: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Role {
    User,
    Assistant,
    System,
}

impl fmt::Display for Role {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Role::User => write!(f, "user"),
            Role::Assistant => write!(f, "assistant"),
            Role::System => write!(f, "system"),
        }
    }
}

/// Response from Anthropic API.
#[derive(Debug, Clone, Deserialize)]
pub struct Response {
    pub content: Vec<ContentBlock>,
    pub usage: Usage,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ContentBlock {
    #[serde(rename = "type")]
    pub block_type: String,
    pub text: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Usage {
    #[serde(rename = "input_tokens")]
    pub input_tokens: i32,
    #[serde(rename = "output_tokens")]
    pub output_tokens: i32,
}

/// Anthropic API client.
#[derive(Debug, Clone)]
pub struct AnthropicClient {
    api_key: String,
    client: reqwest::Client,
}

impl AnthropicClient {
    /// Create a new client.
    pub fn new() -> Result<Self> {
        let api_key = std::env::var("ANTHROPIC_API_KEY").map_err(|_| ApiKeyError)?;

        Ok(Self {
            api_key,
            client: reqwest::Client::new(),
        })
    }

    /// Make an API call.
    pub async fn message(
        &self,
        model: &str,
        messages: &[Message],
        max_tokens: i32,
        temperature: Option<f32>,
    ) -> Result<Response> {
        let mut body = serde_json::json!({
            "model": model,
            "messages": messages.iter().map(|m| serde_json::json!({
                "role": m.role.to_string(),
                "content": m.content
            })).collect::<Vec<_>>(),
            "max_tokens": max_tokens,
        });

        if let Some(temp) = temperature {
            body["temperature"] = serde_json::json!(temp);
        }

        let response = self
            .client
            .post("https://api.anthropic.com/v1/messages")
            .header("x-api-key", &self.api_key)
            .header("anthropic-version", "2023-06-01")
            .header("content-type", "application/json")
            .json(&body)
            .send()
            .await
            .context("Failed to send request to Anthropic")?;

        if !response.status().is_success() {
            let text = response
                .text()
                .await
                .context("Failed to read error response")?;
            anyhow::bail!("Anthropic API error: {text}");
        }

        response
            .json()
            .await
            .context("Failed to parse Anthropic response")
    }
}

impl Default for AnthropicClient {
    fn default() -> Self {
        Self::new().expect("ANTHROPIC_API_KEY must be set")
    }
}

/// Parse JSON from AI response text.
pub fn parse_json_response<T: for<'de> Deserialize<'de>>(text: &str) -> Result<T> {
    let cleaned = if text.contains("```json") {
        let start = text.find("```json").map_or(0, |i| i + 7);
        let end = text[start..]
            .find("```")
            .map_or(text.len(), |i| start + i);
        text[start..end].trim()
    } else if text.contains("```") {
        let start = text.find("```").map_or(0, |i| i + 3);
        let end = text[start..]
            .find("```")
            .map_or(text.len(), |i| start + i);
        text[start..end].trim()
    } else {
        text.trim()
    };

    serde_json::from_str(cleaned).context("Failed to parse AI response as JSON")
}
