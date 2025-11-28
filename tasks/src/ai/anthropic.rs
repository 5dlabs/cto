//! Anthropic Claude AI provider implementation.

use async_trait::async_trait;
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
    "claude-sonnet-4-20250514",
    "claude-3-5-sonnet-20241022",
    "claude-3-5-haiku-20241022",
    "claude-3-opus-20240229",
    "claude-3-sonnet-20240229",
    "claude-3-haiku-20240307",
];

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
}

/// Anthropic API response content
#[derive(Debug, Deserialize)]
struct AnthropicContent {
    #[serde(rename = "type")]
    content_type: String,
    text: String,
}

/// Anthropic API usage
#[derive(Debug, Deserialize)]
struct AnthropicUsage {
    input_tokens: u32,
    output_tokens: u32,
}

/// Anthropic API response
#[derive(Debug, Deserialize)]
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
    fn name(&self) -> &str {
        "anthropic"
    }

    fn api_key_env_var(&self) -> &str {
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
        let api_key = self.api_key.as_ref().ok_or_else(|| {
            TasksError::Ai("ANTHROPIC_API_KEY not set".to_string())
        })?;

        let (system, converted_messages) = self.convert_messages(messages);

        let request = AnthropicRequest {
            model: model.to_string(),
            messages: converted_messages,
            max_tokens: options.max_tokens.unwrap_or(4096),
            system,
            temperature: options.temperature,
            stop_sequences: options.stop_sequences.clone(),
        };

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
        let body = response
            .text()
            .await
            .map_err(|e| TasksError::Ai(format!("Failed to read response: {}", e)))?;

        if !status.is_success() {
            // Try to parse error response
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

        let api_response: AnthropicResponse = serde_json::from_str(&body)
            .map_err(|e| TasksError::Ai(format!("Failed to parse response: {}", e)))?;

        // Extract text from content blocks
        let text = api_response
            .content
            .iter()
            .filter(|c| c.content_type == "text")
            .map(|c| c.text.as_str())
            .collect::<Vec<_>>()
            .join("");

        Ok(AIResponse {
            text,
            usage: TokenUsage {
                input_tokens: api_response.usage.input_tokens,
                output_tokens: api_response.usage.output_tokens,
                total_tokens: api_response.usage.input_tokens + api_response.usage.output_tokens,
            },
            model: api_response.model,
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
        assert!(provider.supports_model("claude-sonnet-4-20250514"));
        assert!(provider.supports_model("claude-3-opus-20240229"));
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

