//! AI Provider trait and common types.
//!
//! Defines the interface that all AI providers must implement.

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::errors::TasksResult;

/// Default thinking budget for extended thinking mode (10K tokens).
pub const DEFAULT_THINKING_BUDGET: u32 = 10_000;

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
    /// Enable extended thinking for more complex reasoning
    pub extended_thinking: bool,
    /// Budget in tokens for extended thinking
    pub thinking_budget: Option<u32>,
    /// Path to MCP config file
    pub mcp_config: Option<String>,
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

    // Handle cases where AI includes leading prose before JSON block
    // The JSON may contain embedded ``` markers (code examples), so we need to find
    // the LAST ``` which closes the JSON block, not the first one we encounter
    let json_text = if let Some(json_start) = text.find("```json") {
        let after_marker = &text[json_start + "```json".len()..];
        // Find the LAST ``` which closes the block (not embedded code examples)
        if let Some(end_idx) = after_marker.rfind("\n```") {
            after_marker[..end_idx].trim()
        } else if let Some(end_idx) = after_marker.rfind("```") {
            after_marker[..end_idx].trim()
        } else {
            after_marker.trim()
        }
    } else if let Some(code_start) = text.find("```\n{") {
        // Handle ```\n{ pattern (code block without language tag)
        let after_marker = &text[code_start + "```\n".len()..];
        if let Some(end_idx) = after_marker.rfind("\n```") {
            after_marker[..end_idx].trim()
        } else if let Some(end_idx) = after_marker.rfind("```") {
            after_marker[..end_idx].trim()
        } else {
            after_marker.trim()
        }
    } else if let Some(first_brace) = text.find('{') {
        // Fallback: find the first { and assume JSON starts there
        // Find matching closing brace by counting nesting
        let json_part = &text[first_brace..];
        let mut depth = 0;
        let mut end_idx = json_part.len();
        for (i, c) in json_part.chars().enumerate() {
            match c {
                '{' => depth += 1,
                '}' => {
                    depth -= 1;
                    if depth == 0 {
                        end_idx = i + 1;
                        break;
                    }
                }
                _ => {}
            }
        }
        &json_part[..end_idx]
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
