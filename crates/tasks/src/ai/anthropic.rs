//! Anthropic Claude AI provider implementation.

use async_trait::async_trait;
use futures::StreamExt;
use reqwest::Client;
use serde::{Deserialize, Serialize};

use crate::errors::{TasksError, TasksResult};

use super::provider::{AIMessage, AIProvider, AIResponse, AIRole, GenerateOptions, TokenUsage};

/// Anthropic API endpoint
const ANTHROPIC_API_URL: &str = "https://api.anthropic.com/v1/messages";

/// Anthropic API version
const ANTHROPIC_VERSION: &str = "2023-06-01";

/// Default model
const DEFAULT_MODEL: &str = "claude-sonnet-4-20250514";

/// Supported Anthropic models
const SUPPORTED_MODELS: &[&str] = &[
    // Claude 4.5 models (latest)
    "claude-opus-4-5-20251101",
    "claude-opus-4-5-20250929", // Alias: common typo (Sonnet date used for Opus)
    "claude-sonnet-4-5-20250929",
    // Short names for convenience
    "opus",
    "sonnet",
    "haiku",
    // Claude 4.1 models
    "claude-opus-4-1-20250805",
    // Claude 4 models
    "claude-sonnet-4-20250514",
    // Claude 3.5 models (deprecated, but kept for backwards compatibility tests)
    // Note: claude-3-5-sonnet-20241022 was removed from Anthropic API
    "claude-3-5-haiku-20241022",
    // Claude 3 models
    "claude-3-opus-20240229",
    "claude-3-sonnet-20240229",
    "claude-3-haiku-20240307",
];

/// Normalize model name to the canonical API model name.
/// Maps short names and common aliases to their correct API identifiers.
fn normalize_model(model: &str) -> &str {
    match model {
        // Short names → latest versions
        // Also handles common typo: using Sonnet 4.5 date for Opus 4.5
        "opus" | "claude-opus-4-5-20250929" => "claude-opus-4-5-20251101",
        "sonnet" => "claude-sonnet-4-5-20250929",
        "haiku" => "claude-3-5-haiku-20241022",
        // Everything else passes through
        _ => model,
    }
}

/// Anthropic API request message
#[derive(Debug, Serialize)]
struct AnthropicMessage {
    role: String,
    content: String,
}

/// Anthropic API request
#[derive(Debug, Serialize)]
struct AnthropicRequest {
    model: String,
    messages: Vec<AnthropicMessage>,
    max_tokens: u32,
    #[serde(skip_serializing_if = "Option::is_none")]
    system: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    temperature: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    stop_sequences: Option<Vec<String>>,
    /// Enable streaming for progress output
    #[serde(skip_serializing_if = "std::ops::Not::not")]
    stream: bool,
}

/// Anthropic API response content
#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct AnthropicContent {
    #[serde(rename = "type")]
    content_type: String,
    text: String,
}

/// Anthropic API usage
#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct AnthropicUsage {
    input_tokens: u32,
    output_tokens: u32,
}

/// Anthropic API response
#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct AnthropicResponse {
    content: Vec<AnthropicContent>,
    model: String,
    usage: AnthropicUsage,
}

/// Anthropic API error
#[derive(Debug, Deserialize)]
struct AnthropicError {
    #[serde(rename = "type")]
    error_type: String,
    message: String,
}

/// Anthropic API error response
#[derive(Debug, Deserialize)]
struct AnthropicErrorResponse {
    error: AnthropicError,
}

/// Streaming event types
#[derive(Debug, Deserialize)]
#[serde(tag = "type")]
#[allow(dead_code)]
enum StreamEvent {
    #[serde(rename = "message_start")]
    MessageStart { message: StreamMessage },
    #[serde(rename = "content_block_start")]
    ContentBlockStart {
        index: usize,
        content_block: ContentBlock,
    },
    #[serde(rename = "content_block_delta")]
    ContentBlockDelta { index: usize, delta: ContentDelta },
    #[serde(rename = "content_block_stop")]
    ContentBlockStop { index: usize },
    #[serde(rename = "message_delta")]
    MessageDelta {
        delta: MessageDeltaContent,
        usage: Option<StreamUsage>,
    },
    #[serde(rename = "message_stop")]
    MessageStop,
    #[serde(rename = "ping")]
    Ping,
    #[serde(rename = "error")]
    Error { error: AnthropicError },
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct StreamMessage {
    id: String,
    model: String,
    usage: StreamUsage,
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct StreamUsage {
    input_tokens: u32,
    #[serde(default)]
    output_tokens: u32,
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct ContentBlock {
    #[serde(rename = "type")]
    block_type: String,
    #[serde(default)]
    text: String,
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct ContentDelta {
    #[serde(rename = "type")]
    delta_type: String,
    #[serde(default)]
    text: String,
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct MessageDeltaContent {
    #[serde(default)]
    stop_reason: Option<String>,
}

/// Anthropic Claude provider.
pub struct AnthropicProvider {
    client: Client,
    api_key: Option<String>,
    base_url: String,
}

impl AnthropicProvider {
    /// Create a new Anthropic provider with an API key.
    pub fn new(api_key: impl Into<String>) -> Self {
        Self {
            client: Client::new(),
            api_key: Some(api_key.into()),
            base_url: ANTHROPIC_API_URL.to_string(),
        }
    }

    /// Create from environment variable.
    pub fn from_env() -> TasksResult<Self> {
        let api_key = std::env::var("ANTHROPIC_API_KEY").ok();
        Ok(Self {
            client: Client::new(),
            api_key,
            base_url: ANTHROPIC_API_URL.to_string(),
        })
    }

    /// Set a custom base URL.
    pub fn with_base_url(mut self, url: impl Into<String>) -> Self {
        self.base_url = url.into();
        self
    }

    /// Convert messages to Anthropic format, extracting system message.
    fn convert_messages(&self, messages: &[AIMessage]) -> (Option<String>, Vec<AnthropicMessage>) {
        let mut system = None;
        let mut converted = Vec::new();

        for msg in messages {
            match msg.role {
                AIRole::System => {
                    // Anthropic uses a separate system field
                    system = Some(msg.content.clone());
                }
                AIRole::User => {
                    converted.push(AnthropicMessage {
                        role: "user".to_string(),
                        content: msg.content.clone(),
                    });
                }
                AIRole::Assistant => {
                    converted.push(AnthropicMessage {
                        role: "assistant".to_string(),
                        content: msg.content.clone(),
                    });
                }
            }
        }

        (system, converted)
    }
}

#[async_trait]
impl AIProvider for AnthropicProvider {
    fn name(&self) -> &'static str {
        "anthropic"
    }

    fn api_key_env_var(&self) -> &'static str {
        "ANTHROPIC_API_KEY"
    }

    fn is_configured(&self) -> bool {
        self.api_key.is_some()
    }

    fn supported_models(&self) -> Vec<&str> {
        SUPPORTED_MODELS.to_vec()
    }

    async fn generate_text(
        &self,
        model: &str,
        messages: &[AIMessage],
        options: &GenerateOptions,
    ) -> TasksResult<AIResponse> {
        let api_key = self
            .api_key
            .as_ref()
            .ok_or_else(|| TasksError::Ai("ANTHROPIC_API_KEY not set".to_string()))?;

        // Normalize model name (handles short names and common aliases)
        let normalized_model = normalize_model(model);

        let (system, converted_messages) = self.convert_messages(messages);

        // Use streaming for progress output
        let request = AnthropicRequest {
            model: normalized_model.to_string(),
            messages: converted_messages,
            max_tokens: options.max_tokens.unwrap_or(4096),
            system,
            temperature: options.temperature,
            stop_sequences: options.stop_sequences.clone(),
            stream: true,
        };

        tracing::info!("Calling Claude API (streaming)...");

        let response = self
            .client
            .post(&self.base_url)
            .header("x-api-key", api_key)
            .header("anthropic-version", ANTHROPIC_VERSION)
            .header("content-type", "application/json")
            .json(&request)
            .send()
            .await
            .map_err(|e| TasksError::Ai(format!("Anthropic API request failed: {}", e)))?;

        let status = response.status();

        if !status.is_success() {
            let body = response
                .text()
                .await
                .map_err(|e| TasksError::Ai(format!("Failed to read response: {}", e)))?;

            if let Ok(error_response) = serde_json::from_str::<AnthropicErrorResponse>(&body) {
                return Err(TasksError::Ai(format!(
                    "Anthropic API error: {} - {}",
                    error_response.error.error_type, error_response.error.message
                )));
            }
            return Err(TasksError::Ai(format!(
                "Anthropic API error ({}): {}",
                status, body
            )));
        }

        // Process streaming response
        let mut full_text = String::new();
        let mut input_tokens = 0u32;
        let mut output_tokens = 0u32;
        let mut response_model = model.to_string();
        let mut char_count = 0usize;
        let mut last_progress = 0usize;

        let mut stream = response.bytes_stream();
        let mut buffer = String::new();

        while let Some(chunk_result) = stream.next().await {
            let chunk =
                chunk_result.map_err(|e| TasksError::Ai(format!("Stream read error: {}", e)))?;

            let chunk_str = String::from_utf8_lossy(&chunk);
            buffer.push_str(&chunk_str);

            // Process complete SSE events from buffer
            while let Some(event_end) = buffer.find("\n\n") {
                let event_data = buffer[..event_end].to_string();
                buffer = buffer[event_end + 2..].to_string();

                // Parse SSE event
                for line in event_data.lines() {
                    if let Some(data) = line.strip_prefix("data: ") {
                        if data == "[DONE]" {
                            continue;
                        }

                        if let Ok(event) = serde_json::from_str::<StreamEvent>(data) {
                            match event {
                                StreamEvent::MessageStart { message } => {
                                    response_model = message.model;
                                    input_tokens = message.usage.input_tokens;
                                    tracing::debug!("Input tokens: {}", input_tokens);
                                }
                                StreamEvent::ContentBlockDelta { delta, .. } => {
                                    if delta.delta_type == "text_delta" {
                                        full_text.push_str(&delta.text);
                                        char_count += delta.text.len();

                                        // Log progress every 500 chars
                                        if char_count - last_progress >= 500 {
                                            tracing::trace!("Generated {} chars...", char_count);
                                            last_progress = char_count;
                                        }
                                    }
                                }
                                StreamEvent::MessageDelta { usage: Some(u), .. } => {
                                    output_tokens = u.output_tokens;
                                }
                                StreamEvent::MessageStop => {
                                    tracing::debug!(
                                        "Output tokens: {}, chars: {}",
                                        output_tokens,
                                        char_count
                                    );
                                }
                                StreamEvent::Error { error } => {
                                    return Err(TasksError::Ai(format!(
                                        "Stream error: {} - {}",
                                        error.error_type, error.message
                                    )));
                                }
                                _ => {}
                            }
                        }
                    }
                }
            }
        }

        Ok(AIResponse {
            text: full_text,
            usage: TokenUsage {
                input_tokens,
                output_tokens,
                total_tokens: input_tokens + output_tokens,
            },
            model: response_model,
            provider: "anthropic".to_string(),
        })
    }
}

impl Default for AnthropicProvider {
    fn default() -> Self {
        Self::from_env().unwrap_or_else(|_| Self {
            client: Client::new(),
            api_key: None,
            base_url: ANTHROPIC_API_URL.to_string(),
        })
    }
}

/// Get the default Anthropic model.
pub fn default_model() -> &'static str {
    DEFAULT_MODEL
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_provider_name() {
        let provider = AnthropicProvider::default();
        assert_eq!(provider.name(), "anthropic");
    }

    #[test]
    fn test_supported_models() {
        let provider = AnthropicProvider::default();
        // Claude 4.5 models
        assert!(provider.supports_model("claude-opus-4-5-20251101"));
        assert!(provider.supports_model("claude-sonnet-4-5-20250929"));
        // Claude 4.1 models
        assert!(provider.supports_model("claude-opus-4-1-20250805"));
        // Claude 4 models
        assert!(provider.supports_model("claude-sonnet-4-20250514"));
        // Claude 3 models
        assert!(provider.supports_model("claude-3-opus-20240229"));
        // Non-Anthropic models
        assert!(!provider.supports_model("gpt-4"));
    }

    #[test]
    fn test_message_conversion() {
        let provider = AnthropicProvider::default();
        let messages = vec![
            AIMessage::system("You are a helpful assistant"),
            AIMessage::user("Hello"),
            AIMessage::assistant("Hi there!"),
            AIMessage::user("How are you?"),
        ];

        let (system, converted) = provider.convert_messages(&messages);

        assert_eq!(system, Some("You are a helpful assistant".to_string()));
        assert_eq!(converted.len(), 3);
        assert_eq!(converted[0].role, "user");
        assert_eq!(converted[1].role, "assistant");
        assert_eq!(converted[2].role, "user");
    }
}
