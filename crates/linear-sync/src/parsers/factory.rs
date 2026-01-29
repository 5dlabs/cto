//! Factory Droid CLI stream parser.
//!
//! Parses Factory's JSONL output format which emits events:
//! - `message`: Text messages from assistant
//! - `tool_call`: Tool invocations with `toolName` and `parameters`
//! - `tool_result`: Tool execution results
//! - `result`: Session completion with stats
//!
//! # Example Events
//!
//! ```json
//! {"type":"message","role":"assistant","text":"Running tasks"}
//! {"type":"tool_call","toolName":"Execute","parameters":{"command":"ls"}}
//! {"type":"tool_result","output":"file1.txt\nfile2.txt"}
//! {"type":"result","is_error":false,"result":"Task completed","model":"gpt-5","duration_ms":1000}
//! ```

use serde::Deserialize;
use serde_json::Value;

use super::{
    init_log, ArtifactOperation, InitInfo, ParseResult, ParsedActivity, StreamParser, StreamStats,
};

/// Factory stream event types
#[derive(Debug, Clone, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum FactoryStreamEvent {
    /// Text message from assistant
    Message {
        /// Role (usually "assistant")
        #[serde(default)]
        role: Option<String>,
        /// Message text
        text: Option<String>,
    },
    /// Tool invocation
    ToolCall {
        /// Tool name
        #[serde(rename = "toolName")]
        tool_name: Option<String>,
        /// Tool parameters
        parameters: Option<Value>,
        /// Call ID
        id: Option<String>,
        /// Alternative call ID
        #[serde(rename = "callId")]
        call_id: Option<String>,
    },
    /// Tool execution result
    ToolResult {
        /// Result output
        output: Option<String>,
        /// Whether the result is an error
        #[serde(default)]
        is_error: Option<bool>,
    },
    /// Final session result
    Result {
        /// Whether this is an error result
        #[serde(alias = "isError")]
        is_error: Option<bool>,
        /// Result text
        result: Option<String>,
        /// Alternative: message text
        message: Option<String>,
        /// Model used
        model: Option<String>,
        /// Duration in milliseconds
        #[serde(alias = "durationMs")]
        duration_ms: Option<u64>,
        /// Usage statistics
        usage: Option<UsageStats>,
    },
    /// Error event
    Error {
        /// Error message
        message: Option<String>,
    },
}

/// Usage statistics from result event
#[derive(Debug, Clone, Deserialize)]
pub struct UsageStats {
    /// Input tokens
    #[serde(alias = "inputTokens")]
    input_tokens: Option<u64>,
    /// Output tokens
    #[serde(alias = "outputTokens")]
    output_tokens: Option<u64>,
}

/// State tracking for tool call/result pairing
#[derive(Debug, Default)]
struct ToolState {
    current_tool: Option<String>,
    current_input: Option<String>,
    current_input_json: Option<Value>,
}

/// Factory stream parser implementation
pub struct FactoryParser {
    stats: StreamStats,
    tool_state: ToolState,
}

impl FactoryParser {
    /// Create a new Factory parser
    #[must_use]
    pub fn new() -> Self {
        Self {
            stats: StreamStats::new(),
            tool_state: ToolState::default(),
        }
    }

    /// Parse a Message event
    fn parse_message_event(&self, role: &Option<String>, text: &Option<String>) -> ParseResult {
        // Only process assistant messages
        let is_assistant = role
            .as_ref()
            .is_none_or(|r| r.eq_ignore_ascii_case("assistant"));

        if !is_assistant {
            return ParseResult::empty();
        }

        let Some(text) = text else {
            return ParseResult::empty();
        };

        // Skip short or boilerplate text
        if text.chars().count() <= 30 {
            return ParseResult::empty();
        }

        let display_text = truncate_chars(text, 500);
        ParseResult::with_activity(ParsedActivity::thought(display_text))
    }

    /// Parse a `ToolCall` event
    fn parse_tool_call_event(
        &mut self,
        tool_name: &Option<String>,
        parameters: &Option<Value>,
    ) -> ParseResult {
        let name = tool_name.clone().unwrap_or_else(|| "tool_call".to_string());
        let input_summary = parameters
            .as_ref()
            .map_or_else(String::new, |v| truncate_chars(&v.to_string(), 150));

        // Store state for pairing with result
        self.tool_state.current_tool = Some(name.clone());
        self.tool_state.current_input = Some(input_summary.clone());
        self.tool_state.current_input_json.clone_from(parameters);

        ParseResult::with_activity(ParsedActivity::action(&name, &input_summary))
    }

    /// Parse a `ToolResult` event
    fn parse_tool_result_event(
        &mut self,
        output: &Option<String>,
        is_error: Option<bool>,
    ) -> ParseResult {
        let mut result = ParseResult::empty();

        // Get tool info from state
        let tool_name = self
            .tool_state
            .current_tool
            .take()
            .unwrap_or_else(|| "Tool".to_string());
        let tool_input = self.tool_state.current_input.take().unwrap_or_default();
        let tool_input_json = self.tool_state.current_input_json.take();

        let is_err = is_error.unwrap_or(false);
        let status_emoji = if is_err { "❌" } else { "✅" };
        let result_preview = output
            .as_ref()
            .map_or_else(|| "No output".to_string(), |r| truncate_chars(r, 200));

        let formatted_result = format!("{status_emoji} {result_preview}");

        result.activities.push(ParsedActivity::action_complete(
            &tool_name,
            &tool_input,
            formatted_result,
        ));

        // Track file operations
        if !is_err {
            if let Some(ref input_json) = tool_input_json {
                let tool_lower = tool_name.to_lowercase();
                if let Some(path) = extract_path_from_input(input_json) {
                    if tool_lower.contains("write") || tool_lower.contains("create") {
                        result.add_artifact(ArtifactOperation::Create { path });
                    } else if tool_lower.contains("read") || tool_lower.contains("file") {
                        result.add_artifact(ArtifactOperation::Read { path });
                    } else if tool_lower.contains("edit") || tool_lower.contains("modify") {
                        result.add_artifact(ArtifactOperation::Modify {
                            path,
                            summary: "Modified".to_string(),
                        });
                    }
                }
            }
        }

        result
    }

    /// Parse a Result event
    fn parse_result_event(
        &mut self,
        is_error: Option<bool>,
        result: &Option<String>,
        message: &Option<String>,
        model: &Option<String>,
        duration_ms: Option<u64>,
        usage: &Option<UsageStats>,
    ) -> ParseResult {
        // Update stats
        if let Some(ms) = duration_ms {
            self.stats.set_duration(ms);
        }
        if let Some(usage) = usage {
            if let (Some(input), Some(output)) = (usage.input_tokens, usage.output_tokens) {
                self.stats.add_tokens(input, output);
            }
        }
        if let Some(m) = model {
            // Note: stats doesn't have model field, but we could add it
            tracing::debug!(model = %m, "Factory result model");
        }

        let is_err = is_error.unwrap_or(false);

        #[allow(clippy::cast_precision_loss)]
        let duration_secs = duration_ms.map_or(0.0, |ms| ms as f64 / 1000.0);

        let result_text = result.as_ref().or(message.as_ref());
        let summary = if let Some(text) = result_text {
            let preview = truncate_chars(text, 100);
            format!("**Completed** | {duration_secs:.1}s | {preview}")
        } else {
            format!("**Completed** | {duration_secs:.1}s")
        };

        let activity = if is_err {
            ParsedActivity::error(summary)
        } else {
            ParsedActivity::response(summary)
        };

        ParseResult::with_activity(activity)
    }
}

impl Default for FactoryParser {
    fn default() -> Self {
        Self::new()
    }
}

impl StreamParser for FactoryParser {
    fn id(&self) -> &'static str {
        "factory"
    }

    fn can_parse(&self, sample_line: &str) -> bool {
        // Factory has "type":"tool_call" with "toolName" field
        sample_line.contains("\"type\":\"tool_call\"") && sample_line.contains("\"toolName\"")
    }

    fn parse_init_from_log(&self, log_content: &str) -> Option<InitInfo> {
        // Factory uses log file for MCP init info
        init_log::parse_mcp_init_from_log(log_content)
    }

    fn parse_line(&mut self, line: &str) -> ParseResult {
        let Ok(event) = serde_json::from_str::<FactoryStreamEvent>(line) else {
            return ParseResult::empty();
        };

        match event {
            FactoryStreamEvent::Message { role, text } => self.parse_message_event(&role, &text),
            FactoryStreamEvent::ToolCall {
                tool_name,
                parameters,
                ..
            } => self.parse_tool_call_event(&tool_name, &parameters),
            FactoryStreamEvent::ToolResult { output, is_error } => {
                self.parse_tool_result_event(&output, is_error)
            }
            FactoryStreamEvent::Result {
                is_error,
                result,
                message,
                model,
                duration_ms,
                usage,
            } => {
                self.stats.increment_turns();
                self.parse_result_event(is_error, &result, &message, &model, duration_ms, &usage)
            }
            FactoryStreamEvent::Error { message } => {
                let error_msg = message.unwrap_or_else(|| "Unknown error".to_string());
                ParseResult::with_activity(ParsedActivity::error(error_msg))
            }
        }
    }

    fn get_stats(&self) -> StreamStats {
        self.stats.clone()
    }

    fn reset(&mut self) {
        self.stats = StreamStats::new();
        self.tool_state = ToolState::default();
    }
}

/// Truncate a string to a maximum number of characters.
fn truncate_chars(s: &str, max_chars: usize) -> String {
    let chars: Vec<char> = s.chars().collect();
    if chars.len() <= max_chars {
        s.to_string()
    } else {
        let truncated: String = chars[..max_chars].iter().collect();
        format!("{truncated}...")
    }
}

/// Extract file path from tool input JSON.
fn extract_path_from_input(input: &Value) -> Option<String> {
    for field in &["path", "file_path", "filepath", "file", "filename"] {
        if let Some(Value::String(path)) = input.get(field) {
            return Some(path.clone());
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_can_parse() {
        let parser = FactoryParser::new();
        assert!(parser.can_parse(r#"{"type":"tool_call","toolName":"Execute","parameters":{}}"#));
        assert!(!parser.can_parse(r#"{"type":"assistant","message":{}}"#));
    }

    #[test]
    fn test_parse_tool_call() {
        let mut parser = FactoryParser::new();

        let line = r#"{"type":"tool_call","toolName":"Execute","parameters":{"command":"ls"}}"#;
        let result = parser.parse_line(line);
        assert_eq!(result.activities.len(), 1);

        match &result.activities[0] {
            ParsedActivity::Action { name, .. } => {
                assert_eq!(name, "Execute");
            }
            _ => panic!("Expected action activity"),
        }
    }

    #[test]
    fn test_parse_result() {
        let mut parser = FactoryParser::new();

        let line = r#"{"type":"result","is_error":false,"result":"Done","model":"gpt-5","duration_ms":1000,"usage":{"input_tokens":100,"output_tokens":200}}"#;
        let result = parser.parse_line(line);
        assert_eq!(result.activities.len(), 1);

        match &result.activities[0] {
            ParsedActivity::Response { body } => {
                assert!(body.contains("Completed"));
                assert!(body.contains("1.0s"));
            }
            _ => panic!("Expected response activity"),
        }

        let stats = parser.get_stats();
        assert_eq!(stats.duration_ms, 1000);
    }
}
