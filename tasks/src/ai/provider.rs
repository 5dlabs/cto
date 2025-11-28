//! AI Provider trait and common types.
//!
//! Defines the interface that all AI providers must implement.

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::errors::TasksResult;

/// Role of a message in a conversation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum AIRole {
    /// System message (sets context/behavior)
    System,
    /// User message (input)
    User,
    /// Assistant message (AI response)
    Assistant,
}

/// A message in a conversation with an AI model.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AIMessage {
    /// Role of the message sender
    pub role: AIRole,
    /// Content of the message
    pub content: String,
}

impl AIMessage {
    /// Create a new system message.
    pub fn system(content: impl Into<String>) -> Self {
        Self {
            role: AIRole::System,
            content: content.into(),
        }
    }

    /// Create a new user message.
    pub fn user(content: impl Into<String>) -> Self {
        Self {
            role: AIRole::User,
            content: content.into(),
        }
    }

    /// Create a new assistant message.
    pub fn assistant(content: impl Into<String>) -> Self {
        Self {
            role: AIRole::Assistant,
            content: content.into(),
        }
    }
}

/// Token usage information from an AI response.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct TokenUsage {
    /// Number of input tokens
    pub input_tokens: u32,
    /// Number of output tokens
    pub output_tokens: u32,
    /// Total tokens (input + output)
    pub total_tokens: u32,
}

/// Response from an AI model.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AIResponse {
    /// Generated text content
    pub text: String,
    /// Token usage information
    pub usage: TokenUsage,
    /// Model that generated the response
    pub model: String,
    /// Provider that generated the response
    pub provider: String,
}

/// Options for text generation.
#[derive(Debug, Clone, Default)]
pub struct GenerateOptions {
    /// Temperature for sampling (0.0 to 1.0)
    pub temperature: Option<f32>,
    /// Maximum tokens to generate
    pub max_tokens: Option<u32>,
    /// Stop sequences
    pub stop_sequences: Option<Vec<String>>,
    /// Whether to request JSON output
    pub json_mode: bool,
    /// Schema name for structured output
    pub schema_name: Option<String>,
}

/// Configuration for an AI provider.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProviderConfig {
    /// API key for authentication
    pub api_key: String,
    /// Base URL for the API (optional, for custom endpoints)
    pub base_url: Option<String>,
    /// Additional headers
    #[serde(default)]
    pub headers: HashMap<String, String>,
    /// Default model to use
    pub default_model: Option<String>,
}

/// Trait for AI providers.
///
/// All AI providers (Anthropic, OpenAI, etc.) must implement this trait.
#[async_trait]
pub trait AIProvider: Send + Sync {
    /// Get the provider name (e.g., "anthropic", "openai").
    fn name(&self) -> &'static str;

    /// Get the environment variable name for the API key.
    fn api_key_env_var(&self) -> &'static str;

    /// Check if the provider is configured (has API key).
    fn is_configured(&self) -> bool;

    /// Get the list of supported models.
    fn supported_models(&self) -> Vec<&str>;

    /// Check if a model is supported.
    fn supports_model(&self, model: &str) -> bool {
        self.supported_models().contains(&model)
    }

    /// Generate text from messages.
    async fn generate_text(
        &self,
        model: &str,
        messages: &[AIMessage],
        options: &GenerateOptions,
    ) -> TasksResult<AIResponse>;
}

/// Generate a structured object from an AI response.
///
/// This is a standalone function rather than a trait method because
/// generic methods are not dyn-compatible.
pub fn parse_ai_response<T: for<'de> Deserialize<'de>>(response: &AIResponse) -> TasksResult<T> {
    // Try to extract JSON from the response text
    let text = response.text.trim();

    // Sometimes the AI wraps JSON in markdown code blocks
    let json_text = if text.starts_with("```json") {
        text.strip_prefix("```json")
            .and_then(|s| s.strip_suffix("```"))
            .unwrap_or(text)
            .trim()
    } else if text.starts_with("```") {
        text.strip_prefix("```")
            .and_then(|s| s.strip_suffix("```"))
            .unwrap_or(text)
            .trim()
    } else {
        text
    };

    serde_json::from_str(json_text).map_err(|e| crate::errors::TasksError::AiResponseParseError {
        reason: format!("Failed to parse AI response as JSON: {e}. Response: {text}"),
    })
}

/// Builder for constructing AI messages.
#[derive(Debug, Default)]
pub struct MessageBuilder {
    messages: Vec<AIMessage>,
}

impl MessageBuilder {
    /// Create a new message builder.
    pub fn new() -> Self {
        Self::default()
    }

    /// Add a system message.
    pub fn system(mut self, content: impl Into<String>) -> Self {
        self.messages.push(AIMessage::system(content));
        self
    }

    /// Add a user message.
    pub fn user(mut self, content: impl Into<String>) -> Self {
        self.messages.push(AIMessage::user(content));
        self
    }

    /// Add an assistant message.
    pub fn assistant(mut self, content: impl Into<String>) -> Self {
        self.messages.push(AIMessage::assistant(content));
        self
    }

    /// Build the message list.
    pub fn build(self) -> Vec<AIMessage> {
        self.messages
    }
}
