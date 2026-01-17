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

/// Extract the JSON continuation from a prefill response.
///
/// When using the prefill technique (e.g., assistant message starts with `{"tasks":[`),
/// the AI may include explanatory text before OR within the JSON content. This function
/// extracts just the JSON content suitable for reconstruction.
///
/// For example, if the AI returns:
/// ```text
/// I'll continue from where I was cut off...
///
/// {"id":35,"title":"..."},{"id":36,...}]}
/// ```
///
/// This function returns:
/// ```text
/// {"id":35,"title":"..."},{"id":36,...}]}
/// ```
///
/// It also handles cases where the prefill is echoed back by the CLI:
/// ```text
/// {"tasks":[I'll continue from where I was cut off...{"id":35,"title":"..."}]}
/// ```
///
/// In this case, it extracts just the JSON objects after the embedded text.
///
/// If no JSON structure is found, returns the original text trimmed.
///
/// # Note
/// Use `validate_json_continuation` to check if the result is valid JSON content
/// before reconstructing the full JSON structure.
pub fn extract_json_continuation(text: &str) -> String {
    // Prefill constant used to detect echoed prefill from CLI
    const PREFILL: &str = r#"{"tasks":["#;

    let text = text.trim();

    // CRITICAL: Handle echoed prefill first.
    // When the CLI echoes back the prefill, the response looks like:
    // {"tasks":[\n{\n  "expo": ...}]} or {"tasks":[{"id":1,...}]}
    // We need to strip the {"tasks":[ part so it's not doubled when reconstructed.
    let text = if let Some(stripped) = text.strip_prefix(PREFILL) {
        // Strip the prefill and continue processing the remainder
        // The remainder may still have embedded prose before {"id":
        stripped.trim_start()
    } else {
        text
    };

    // Look for JSON inside markdown code blocks first (```json ... ```)
    // This handles cases where the AI wraps the JSON in a code block
    if let Some(json_block_start) = text.find("```json") {
        let after_marker = &text[json_block_start + "```json".len()..];
        if let Some(end_idx) = after_marker.rfind("```") {
            let json_content = after_marker[..end_idx].trim();
            if !json_content.is_empty() {
                return json_content.to_string();
            }
        }
    }

    // Also check for plain code blocks (``` ... ```)
    if let Some(code_block_start) = text.find("```\n") {
        let after_marker = &text[code_block_start + "```\n".len()..];
        if let Some(end_idx) = after_marker.rfind("```") {
            let json_content = after_marker[..end_idx].trim();
            if json_content.starts_with('{') || json_content.starts_with('[') {
                return json_content.to_string();
            }
        }
    }

    // If the response starts with a JSON array `[`, return as-is
    // Arrays are used for subtasks and should be preserved complete
    if text.starts_with('[') {
        return text.to_string();
    }

    // Look for JSON inside markdown code blocks first (```json ... ```)
    // This handles cases where the AI wraps the JSON in a code block
    if let Some(json_block_start) = text.find("```json") {
        let after_marker = &text[json_block_start + "```json".len()..];
        if let Some(end_idx) = after_marker.rfind("```") {
            let json_content = after_marker[..end_idx].trim();
            if !json_content.is_empty() {
                return json_content.to_string();
            }
        }
    }

    // Also check for plain code blocks (``` ... ```)
    if let Some(code_block_start) = text.find("```\n") {
        let after_marker = &text[code_block_start + "```\n".len()..];
        if let Some(end_idx) = after_marker.rfind("```") {
            let json_content = after_marker[..end_idx].trim();
            if json_content.starts_with('{') || json_content.starts_with('[') {
                return json_content.to_string();
            }
        }
    }

    // Handle the case where the CLI echoes back the prefill with embedded text.
    // For example: {"tasks":[I'll continue...{"id":35,"title":"..."}]}
    // We need to find the FIRST actual JSON object ({"id":) and return from there.
    //
    // This check handles both:
    // 1. Text that starts with { but has embedded prose (prefill echo case)
    // 2. Text that starts with prose followed by JSON (normal case)
    // 3. Text that starts with { but is WRONG structure (e.g., {"expo":... instead of {"id":...)
    if let Some(json_start) = text.find(r#"{"id":"#).or_else(|| text.find(r#"{"id"#)) {
        // Only use this extraction if:
        // - Text doesn't start with { (prose before JSON)
        // - OR text starts with { but json_start > 0 (embedded prose after { like {"tasks":[text{"id":...)
        if !text.starts_with('{') || json_start > 0 {
            return text[json_start..].to_string();
        }
    }

    // If text starts with { but doesn't have "id" key, it might be hallucinated content
    // (e.g., AI outputting {"expo":...} instead of task objects)
    // In this case, we MUST find {"id": or the content is invalid
    if text.starts_with('{') {
        // Check if first key is "id" - if so, it's valid task content
        // Patterns: {"id": or {\n  "id": or { "id":
        let trimmed = text.trim_start_matches('{').trim_start();
        if trimmed.starts_with("\"id\"") {
            return text.to_string();
        }
        // First key is NOT "id" - this is hallucinated content (e.g., {"expo":...})
        // Look for the first {"id": in the entire text
        if let Some(json_start) = text.find(r#"{"id":"#).or_else(|| text.find(r#"{"id"#)) {
            return text[json_start..].to_string();
        }
        // No valid task objects found - return original and let caller handle the error
        return text.to_string();
    }

    // Fallback: look for any JSON object start
    if let Some(first_brace) = text.find('{') {
        return text[first_brace..].to_string();
    }

    // No JSON found, return original
    text.to_string()
}

/// Validates that extracted JSON continuation contains valid task objects.
///
/// This function checks if the result from `extract_json_continuation` is valid
/// JSON content that can be used to reconstruct a tasks array. It returns an
/// error if the content is pure prose (common when AI outputs a summary instead
/// of actual JSON).
///
/// # Arguments
/// * `content` - The string returned from `extract_json_continuation`
///
/// # Returns
/// * `Ok(())` if the content appears to be valid JSON array content
/// * `Err(...)` if the content is invalid (prose, summary, etc.)
///
/// # Example
/// ```ignore
/// let json_content = extract_json_continuation(&response.text);
/// validate_json_continuation(&json_content)?;
/// let full_json = format!(r#"{{"tasks":[{json_content}"#);
/// ```
pub fn validate_json_continuation(content: &str) -> TasksResult<()> {
    let content = content.trim();

    // Empty content is invalid
    if content.is_empty() {
        return Err(crate::errors::TasksError::Ai(
            "AI returned empty response - no task JSON generated".to_string(),
        ));
    }

    // Valid JSON array content should start with:
    // 1. `{` for a JSON object (task)
    // 2. `]` for an empty array (closing bracket from prefill)
    // 3. `]}` for an empty response
    let first_char = content.chars().next().unwrap_or(' ');

    if first_char == '{' || first_char == ']' {
        // Looks like JSON - do a quick sanity check
        if first_char == '{' {
            // The content after `{` should start with `"id"` for valid task objects
            let after_brace = content.trim_start_matches('{').trim_start();
            if !after_brace.starts_with("\"id\"") {
                // Content starts with { but first key is not "id" - this is invalid
                // Note: We do NOT accept nested {"id": fields (e.g., {"wrapper":{"id":1,...}})
                // because they cannot deserialize to GeneratedTask which requires "id" as a
                // direct top-level field
                return Err(crate::errors::TasksError::Ai(format!(
                    "AI response does not contain valid task objects. \
                     Expected JSON array of tasks with 'id' as the first field, but got different structure. \
                     First 200 chars: {}...",
                    &content.chars().take(200).collect::<String>()
                )));
            }
        } else {
            // Content starts with `]` - this should be `]}` to close both the array and object
            // When reconstructed as `{"tasks":[` + content, we need `]}` to produce valid JSON
            // Accepting just `]` would produce `{"tasks":[]` which is invalid (missing `}`)
            let trimmed = content.trim();
            if trimmed != "]}" {
                return Err(crate::errors::TasksError::Ai(format!(
                    "AI returned incomplete JSON structure. \
                     Expected '}}]' to close the tasks array, but got: {}...",
                    &content.chars().take(50).collect::<String>()
                )));
            }
        }
        Ok(())
    } else {
        // Content starts with something else - likely prose
        Err(crate::errors::TasksError::Ai(format!(
            "AI returned a summary or explanation instead of JSON task data. \
             The model should output only JSON array contents. \
             First 200 chars: {}...",
            &content.chars().take(200).collect::<String>()
        )))
    }
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
    //
    // IMPORTANT: Check for ```json FIRST, even if text starts with { or [
    // This handles cases where the prefill technique produces:
    // {"tasks":[explanatory text...```json{actual json}```]}
    let json_text = if text.contains("```json") {
        // Find the ```json block and extract its contents
        let json_start = text.find("```json").unwrap();
        let after_marker = &text[json_start + "```json".len()..];
        // Find the LAST ``` which closes the block (not embedded code examples)
        if let Some(end_idx) = after_marker.rfind("\n```") {
            after_marker[..end_idx].trim()
        } else if let Some(end_idx) = after_marker.rfind("```") {
            after_marker[..end_idx].trim()
        } else {
            after_marker.trim()
        }
    } else if text.contains("```\n{") {
        // Handle ```\n{ pattern (code block without language tag)
        let code_start = text.find("```\n{").unwrap();
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_json_continuation_clean_json() {
        // Already clean JSON should pass through unchanged
        let input = r#"{"id":35,"title":"Test"}"#;
        assert_eq!(extract_json_continuation(input), input);
    }

    #[test]
    fn test_extract_json_continuation_with_leading_text() {
        // Handles case where AI includes explanatory text before JSON
        let input = r#"I'll continue from where I was cut off...

{"id":35,"title":"Test"}"#;
        assert_eq!(
            extract_json_continuation(input),
            r#"{"id":35,"title":"Test"}"#
        );
    }

    #[test]
    fn test_extract_json_continuation_with_extended_thinking_prefix() {
        // Common case from extended thinking models
        let input = r#"Based on the PRD requirements, I'll generate the following tasks:

{"id":1,"title":"Setup project structure"}"#;
        assert_eq!(
            extract_json_continuation(input),
            r#"{"id":1,"title":"Setup project structure"}"#
        );
    }

    #[test]
    fn test_extract_json_continuation_array_start() {
        // Array continuation (for subtasks response)
        let input = r#"[{"id":1,"title":"Subtask 1"},{"id":2,"title":"Subtask 2"}]"#;
        assert_eq!(extract_json_continuation(input), input);
    }

    #[test]
    fn test_extract_json_continuation_no_json() {
        // No JSON found - returns original text
        let input = "This is just plain text with no JSON";
        assert_eq!(extract_json_continuation(input), input);
    }

    #[test]
    fn test_extract_json_continuation_real_world_error() {
        // The actual error case from the intake logs
        let input = r#"I'll continue from where I was cut off, completing the gRPC service handlers and remaining tasks.

{"id":35,"title":"Implement gRPC Service Handlers"}"#;
        assert_eq!(
            extract_json_continuation(input),
            r#"{"id":35,"title":"Implement gRPC Service Handlers"}"#
        );
    }

    #[test]
    fn test_extract_json_continuation_json_code_block() {
        // AI wraps JSON in a markdown code block
        let input = r#"Here is the continuation:

```json
{"id":1,"title":"Test task"}
```"#;
        assert_eq!(
            extract_json_continuation(input),
            r#"{"id":1,"title":"Test task"}"#
        );
    }

    #[test]
    fn test_extract_json_continuation_plain_code_block() {
        // AI wraps JSON in a plain code block without language tag
        let input = r#"Continuing the tasks:

```
{"id":1,"title":"Test task"}
```"#;
        assert_eq!(
            extract_json_continuation(input),
            r#"{"id":1,"title":"Test task"}"#
        );
    }

    #[test]
    fn test_extract_json_continuation_code_block_with_array() {
        // AI wraps JSON array in code block (subtasks case)
        let input = r#"Here are the subtasks:

```json
[{"id":1,"title":"Subtask 1"},{"id":2,"title":"Subtask 2"}]
```"#;
        assert_eq!(
            extract_json_continuation(input),
            r#"[{"id":1,"title":"Subtask 1"},{"id":2,"title":"Subtask 2"}]"#
        );
    }

    #[test]
    fn test_extract_json_continuation_prefill_echo_with_embedded_text() {
        // Critical bug fix: handles case where CLI echoes back the prefill
        // with explanatory text embedded INSIDE the JSON structure
        // e.g., {"tasks":[I'll continue from where...{"id":35,...}]}
        //
        // The intake code does: full_json = format!(r#"{{"tasks":[{json_content}"#)
        // So we need extract_json_continuation to return just the JSON objects
        // when the input looks like: I'll continue...{"id":35,"title":"..."}]}
        let input = r#"I'll continue from where I was cut off, completing the gRPC service handlers and remaining tasks.

{"id":35,"title":"Implement gRPC Service Handlers"},{"id":36,"title":"Another task"}]}"#;
        assert_eq!(
            extract_json_continuation(input),
            r#"{"id":35,"title":"Implement gRPC Service Handlers"},{"id":36,"title":"Another task"}]}"#
        );
    }

    #[test]
    fn test_extract_json_continuation_prefill_echo_starts_with_brace() {
        // Edge case: CLI echoes prefill AND response starts with {
        // This simulates: {"tasks":[Some text here{"id":35,...
        // Where the whole thing starts with { but has embedded prose
        //
        // In this case, json_start would be > 0, so we extract from {"id":
        let input = r#"{"tasks":[Some explanatory text{"id":35,"title":"Test task"}]}"#;
        assert_eq!(
            extract_json_continuation(input),
            r#"{"id":35,"title":"Test task"}]}"#
        );
    }

    #[test]
    fn test_extract_json_continuation_clean_json_with_id_unchanged() {
        // Clean JSON that starts with {"id": should pass through unchanged
        let input = r#"{"id":35,"title":"Test task"}"#;
        assert_eq!(extract_json_continuation(input), input);
    }

    #[test]
    fn test_extract_json_continuation_clean_array_unchanged() {
        // Clean JSON array should pass through unchanged
        let input = r#"[{"id":1},{"id":2}]"#;
        assert_eq!(extract_json_continuation(input), input);
    }

    #[test]
    fn test_extract_json_continuation_echoed_prefill_with_valid_tasks() {
        // Critical bug fix: when CLI echoes back the prefill {"tasks":[
        // we need to strip it so it doesn't get doubled in reconstruction
        let input = r#"{"tasks":[{"id":1,"title":"Task 1"},{"id":2,"title":"Task 2"}]}"#;
        assert_eq!(
            extract_json_continuation(input),
            r#"{"id":1,"title":"Task 1"},{"id":2,"title":"Task 2"}]}"#
        );
    }

    #[test]
    fn test_extract_json_continuation_echoed_prefill_with_hallucinated_content() {
        // Bug: AI echoes prefill but hallucinates wrong content (expo config instead of tasks)
        // The response starts with {"tasks":[ (prefill) then wrong JSON follows
        // We strip the prefill so caller can try to parse what remains
        let input = r#"{"tasks":[
{
  "expo": {
    "name": "AlertHub"
  }
}]}"#;
        // After stripping {"tasks":[, we get the expo content (which will fail parsing as Task)
        // But at least it won't have double {"tasks":[ prefix
        let result = extract_json_continuation(input);
        assert!(
            !result.starts_with(r#"{"tasks":["#),
            "Should strip echoed prefill"
        );
        assert!(
            result.contains("expo"),
            "Should preserve the content after prefill"
        );
    }

    #[test]
    fn test_extract_json_continuation_echoed_prefill_with_newlines() {
        // CLI may echo prefill with newlines in the continuation
        let input = r#"{"tasks":[
{"id":1,"title":"Task 1"}
]}"#;
        let result = extract_json_continuation(input);
        assert!(
            result.starts_with(r#"{"id":1"#),
            "Should strip prefill and return task JSON"
        );
    }

    // Tests for validate_json_continuation

    #[test]
    fn test_validate_json_continuation_valid_task_object() {
        // Valid task object starting with {"id":
        let content = r#"{"id":1,"title":"Setup project"}"#;
        assert!(validate_json_continuation(content).is_ok());
    }

    #[test]
    fn test_validate_json_continuation_valid_task_array() {
        // Multiple tasks
        let content = r#"{"id":1,"title":"Task 1"},{"id":2,"title":"Task 2"}]}"#;
        assert!(validate_json_continuation(content).is_ok());
    }

    #[test]
    fn test_validate_json_continuation_empty_array() {
        // Empty array (closing bracket from prefill)
        // Must be exactly `]}` to close both the array and object when reconstructed
        let content = "]}";
        assert!(validate_json_continuation(content).is_ok());
    }

    #[test]
    fn test_validate_json_continuation_incomplete_closing() {
        // Just `]` without the closing `}` would produce invalid JSON when reconstructed
        // `{"tasks":[` + `]` = `{"tasks":[]` (missing closing brace)
        let content = "]";
        let result = validate_json_continuation(content);
        assert!(
            result.is_err(),
            "Should reject incomplete closing bracket without }}"
        );
        let err = result.unwrap_err().to_string();
        assert!(
            err.contains("incomplete"),
            "Error should mention incomplete: {err}"
        );
    }

    #[test]
    fn test_validate_json_continuation_extra_closing() {
        // Extra brackets would produce invalid JSON
        // `{"tasks":[` + `]}]` = `{"tasks":[]}]` (extra bracket)
        let content = "]}]";
        let result = validate_json_continuation(content);
        assert!(
            result.is_err(),
            "Should reject content with extra closing brackets"
        );
    }

    #[test]
    fn test_validate_json_continuation_prose_only() {
        // Pure prose - AI returned a summary instead of JSON
        let content = "I've generated the complete task breakdown for the AlertHub PRD. The JSON output contains 50 tasks organized as follows:";
        let result = validate_json_continuation(content);
        assert!(result.is_err());
        let err = result.unwrap_err().to_string();
        assert!(
            err.contains("summary or explanation"),
            "Error should mention summary/explanation: {err}"
        );
    }

    #[test]
    fn test_validate_json_continuation_prose_with_bullets() {
        // Prose with bullet points - common failure mode
        let content = "**Infrastructure (Tasks 1-9):**\n- PostgreSQL, Redis/Valkey, Kafka, MongoDB setup\n- Database schema creation";
        let result = validate_json_continuation(content);
        assert!(result.is_err());
    }

    #[test]
    fn test_validate_json_continuation_empty() {
        // Empty content
        let content = "";
        let result = validate_json_continuation(content);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("empty"));
    }

    #[test]
    fn test_validate_json_continuation_whitespace_only() {
        // Whitespace only
        let content = "   \n\t  ";
        let result = validate_json_continuation(content);
        assert!(result.is_err());
    }

    #[test]
    fn test_validate_json_continuation_wrong_json_structure() {
        // JSON but not task objects (starts with { but no "id" key)
        let content = r#"{"expo":{"name":"app","slug":"app"}}"#;
        let result = validate_json_continuation(content);
        assert!(result.is_err());
        let err = result.unwrap_err().to_string();
        assert!(
            err.contains("valid task objects"),
            "Error should mention valid task objects: {err}"
        );
    }

    #[test]
    fn test_validate_json_continuation_has_nested_id() {
        // Content that has {"id": somewhere inside (nested task)
        // This simulates cases like: {"wrapper":{"id":1,"title":"Task"}}
        // which is INVALID because when reconstructed as {"tasks":[{"wrapper":...}]},
        // it cannot deserialize to ParsePrdResponse (wrapper is not a valid field)
        let content = r#"{"wrapper":{"id":1,"title":"Task"}}"#;
        // This should FAIL because "id" must be a direct top-level field, not nested
        let result = validate_json_continuation(content);
        assert!(
            result.is_err(),
            "Should fail because nested {{\"id\":\" is not valid - id must be top-level"
        );
        let err = result.unwrap_err().to_string();
        assert!(
            err.contains("first field"),
            "Error should mention 'id' must be the first field: {err}"
        );
    }

    #[test]
    fn test_validate_json_continuation_no_nested_id() {
        // Content that doesn't have {"id": anywhere
        let content = r#"{"metadata":{"key":"value"},"name":"test"}"#;
        let result = validate_json_continuation(content);
        assert!(result.is_err(), "Should fail because no {{\"id\": found");
    }
}
