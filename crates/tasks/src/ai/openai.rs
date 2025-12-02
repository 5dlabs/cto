//! OpenAI GPT provider implementation.

use async_trait::async_trait;
use reqwest::Client;
use serde::{Deserialize, Serialize};

use crate::errors::{TasksError, TasksResult};

use super::provider::{AIMessage, AIProvider, AIResponse, AIRole, GenerateOptions, TokenUsage};

/// OpenAI API endpoint
const OPENAI_API_URL: &str = "https://api.openai.com/v1/chat/completions";

/// Default model
const DEFAULT_MODEL: &str = "gpt-4o";

/// Supported OpenAI models
const SUPPORTED_MODELS: &[&str] = &[
    "gpt-4o",
    "gpt-4o-mini",
    "gpt-4-turbo",
    "gpt-4",
    "gpt-3.5-turbo",
    "o1-preview",
    "o1-mini",
];

/// OpenAI API request message
#[derive(Debug, Serialize)]
struct OpenAIMessage {
    role: String,
    content: String,
}

/// OpenAI API response format
#[derive(Debug, Serialize)]
struct ResponseFormat {
    #[serde(rename = "type")]
    format_type: String,
}

/// OpenAI API request
#[derive(Debug, Serialize)]
struct OpenAIRequest {
    model: String,
    messages: Vec<OpenAIMessage>,
    #[serde(skip_serializing_if = "Option::is_none")]
    max_tokens: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    temperature: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    stop: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    response_format: Option<ResponseFormat>,
}

/// OpenAI API response choice message
#[derive(Debug, Deserialize)]
struct OpenAIChoiceMessage {
    content: Option<String>,
}

/// OpenAI API response choice
#[derive(Debug, Deserialize)]
struct OpenAIChoice {
    message: OpenAIChoiceMessage,
    #[allow(dead_code)]
    finish_reason: Option<String>,
}

/// OpenAI API usage
#[derive(Debug, Deserialize)]
#[allow(clippy::struct_field_names)]
struct OpenAIUsage {
    prompt_tokens: u32,
    completion_tokens: u32,
    total_tokens: u32,
}

/// OpenAI API response
#[derive(Debug, Deserialize)]
struct OpenAIResponse {
    choices: Vec<OpenAIChoice>,
    model: String,
    usage: OpenAIUsage,
}

/// OpenAI API error
#[derive(Debug, Deserialize)]
struct OpenAIError {
    message: String,
    #[serde(rename = "type")]
    #[allow(dead_code)]
    error_type: Option<String>,
    #[allow(dead_code)]
    code: Option<String>,
}

/// OpenAI API error response
#[derive(Debug, Deserialize)]
struct OpenAIErrorResponse {
    error: OpenAIError,
}

/// OpenAI GPT provider.
pub struct OpenAIProvider {
    client: Client,
    api_key: Option<String>,
    base_url: String,
}

impl OpenAIProvider {
    /// Create a new OpenAI provider with an API key.
    pub fn new(api_key: impl Into<String>) -> Self {
        Self {
            client: Client::new(),
            api_key: Some(api_key.into()),
            base_url: OPENAI_API_URL.to_string(),
        }
    }

    /// Create from environment variable.
    pub fn from_env() -> TasksResult<Self> {
        let api_key = std::env::var("OPENAI_API_KEY").ok();
        Ok(Self {
            client: Client::new(),
            api_key,
            base_url: OPENAI_API_URL.to_string(),
        })
    }

    /// Set a custom base URL (useful for Azure OpenAI or proxies).
    pub fn with_base_url(mut self, url: impl Into<String>) -> Self {
        self.base_url = url.into();
        self
    }

    /// Convert messages to OpenAI format.
    fn convert_messages(&self, messages: &[AIMessage]) -> Vec<OpenAIMessage> {
        messages
            .iter()
            .map(|msg| OpenAIMessage {
                role: match msg.role {
                    AIRole::System => "system".to_string(),
                    AIRole::User => "user".to_string(),
                    AIRole::Assistant => "assistant".to_string(),
                },
                content: msg.content.clone(),
            })
            .collect()
    }
}

#[async_trait]
impl AIProvider for OpenAIProvider {
    fn name(&self) -> &'static str {
        "openai"
    }

    fn api_key_env_var(&self) -> &'static str {
        "OPENAI_API_KEY"
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
            .ok_or_else(|| TasksError::Ai("OPENAI_API_KEY not set".to_string()))?;

        let converted_messages = self.convert_messages(messages);

        let response_format = if options.json_mode {
            Some(ResponseFormat {
                format_type: "json_object".to_string(),
            })
        } else {
            None
        };

        let request = OpenAIRequest {
            model: model.to_string(),
            messages: converted_messages,
            max_tokens: options.max_tokens,
            temperature: options.temperature,
            stop: options.stop_sequences.clone(),
            response_format,
        };

        let response = self
            .client
            .post(&self.base_url)
            .header("Authorization", format!("Bearer {}", api_key))
            .header("Content-Type", "application/json")
            .json(&request)
            .send()
            .await
            .map_err(|e| TasksError::Ai(format!("OpenAI API request failed: {}", e)))?;

        let status = response.status();
        let body = response
            .text()
            .await
            .map_err(|e| TasksError::Ai(format!("Failed to read response: {}", e)))?;

        if !status.is_success() {
            // Try to parse error response
            if let Ok(error_response) = serde_json::from_str::<OpenAIErrorResponse>(&body) {
                return Err(TasksError::Ai(format!(
                    "OpenAI API error: {}",
                    error_response.error.message
                )));
            }
            return Err(TasksError::Ai(format!(
                "OpenAI API error ({}): {}",
                status, body
            )));
        }

        let api_response: OpenAIResponse = serde_json::from_str(&body)
            .map_err(|e| TasksError::Ai(format!("Failed to parse response: {}", e)))?;

        // Extract text from first choice
        let text = api_response
            .choices
            .first()
            .and_then(|c| c.message.content.clone())
            .unwrap_or_default();

        Ok(AIResponse {
            text,
            usage: TokenUsage {
                input_tokens: api_response.usage.prompt_tokens,
                output_tokens: api_response.usage.completion_tokens,
                total_tokens: api_response.usage.total_tokens,
            },
            model: api_response.model,
            provider: "openai".to_string(),
        })
    }
}

impl Default for OpenAIProvider {
    fn default() -> Self {
        Self::from_env().unwrap_or_else(|_| Self {
            client: Client::new(),
            api_key: None,
            base_url: OPENAI_API_URL.to_string(),
        })
    }
}

/// Get the default OpenAI model.
pub fn default_model() -> &'static str {
    DEFAULT_MODEL
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_provider_name() {
        let provider = OpenAIProvider::default();
        assert_eq!(provider.name(), "openai");
    }

    #[test]
    fn test_supported_models() {
        let provider = OpenAIProvider::default();
        assert!(provider.supports_model("gpt-4o"));
        assert!(provider.supports_model("gpt-4"));
        assert!(!provider.supports_model("claude-3-opus"));
    }

    #[test]
    fn test_message_conversion() {
        let provider = OpenAIProvider::default();
        let messages = vec![
            AIMessage::system("You are a helpful assistant"),
            AIMessage::user("Hello"),
            AIMessage::assistant("Hi there!"),
        ];

        let converted = provider.convert_messages(&messages);

        assert_eq!(converted.len(), 3);
        assert_eq!(converted[0].role, "system");
        assert_eq!(converted[1].role, "user");
        assert_eq!(converted[2].role, "assistant");
    }
}
