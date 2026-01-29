//! Claude CLI stream-json parser.
//!
//! Parses Claude's `stream-json` output format which emits JSONL events:
//! - `system`: Initialization with model and tools
//! - `assistant`: Messages with text or `tool_use` content
//! - `user`: Tool results
//! - `result`: Session completion with stats
//!
//! # Example Events
//!
//! ```json
//! {"type":"system","model":"claude-opus-4","tools":["read_file","write_file"]}
//! {"type":"assistant","message":{"content":[{"type":"tool_use","name":"read_file","input":{"path":"src/lib.rs"}}]}}
//! {"type":"user","tool_use_result":"file contents here..."}
//! {"type":"result","duration_ms":5000,"total_cost_usd":0.05,"num_turns":3}
//! ```

use serde::Deserialize;
use serde_json::Value;

use super::{
    init_log, ArtifactOperation, InitInfo, ParseResult, ParsedActivity, StreamParser, StreamStats,
};

/// Claude stream event types (from `stream-json` output)
#[derive(Debug, Clone, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ClaudeStreamEvent {
    /// System initialization event
    System {
        /// Subtype (usually not used)
        #[serde(default)]
        subtype: Option<String>,
        /// Model name
        model: Option<String>,
        /// Available tools
        tools: Option<Vec<String>>,
        /// Session ID
        #[serde(default)]
        session_id: Option<String>,
    },
    /// Assistant message (may contain text or `tool_use`)
    Assistant {
        /// Message content
        message: Option<AssistantMessage>,
        /// Session ID
        #[serde(default)]
        session_id: Option<String>,
    },
    /// User message (usually tool results)
    User {
        /// Message content
        #[serde(default)]
        message: Option<UserMessage>,
        /// Tool use result text
        tool_use_result: Option<String>,
        /// Session ID
        #[serde(default)]
        session_id: Option<String>,
    },
    /// Final result with stats
    Result {
        /// Subtype (e.g., "error" for failures)
        #[serde(default)]
        subtype: Option<String>,
        /// Duration in milliseconds
        duration_ms: Option<u64>,
        /// Total cost in USD
        total_cost_usd: Option<f64>,
        /// Number of turns
        num_turns: Option<u32>,
        /// Result text
        result: Option<String>,
        /// Session ID
        #[serde(default)]
        session_id: Option<String>,
    },
}

/// Assistant message content
#[derive(Debug, Clone, Deserialize)]
pub struct AssistantMessage {
    /// Content blocks
    pub content: Option<Vec<ContentBlock>>,
}

/// Content block types
#[derive(Debug, Clone, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ContentBlock {
    /// Text content
    Text {
        /// Text body
        text: String,
    },
    /// Tool use request
    ToolUse {
        /// Tool name
        name: String,
        /// Tool input (JSON)
        input: Option<Value>,
        /// Tool use ID
        #[serde(default)]
        id: Option<String>,
    },
    /// Tool result (within assistant message)
    ToolResult {
        /// Tool use ID this result is for
        #[serde(default)]
        tool_use_id: Option<String>,
        /// Result content
        content: Option<String>,
    },
}

/// User message content
#[derive(Debug, Clone, Deserialize)]
pub struct UserMessage {
    /// Content blocks
    pub content: Option<Vec<ContentBlock>>,
}

/// State tracking for tool call/result pairing
#[derive(Debug, Default)]
struct ToolState {
    current_tool: Option<String>,
    current_input: Option<String>,
    current_input_json: Option<Value>,
}

/// Claude stream parser implementation
pub struct ClaudeParser {
    stats: StreamStats,
    tool_state: ToolState,
}

impl ClaudeParser {
    /// Create a new Claude parser
    #[must_use]
    pub fn new() -> Self {
        Self {
            stats: StreamStats::new(),
            tool_state: ToolState::default(),
        }
    }

    /// Parse a System event into init info
    fn parse_system_event(
        &mut self,
        model: &Option<String>,
        tools: &Option<Vec<String>>,
    ) -> ParseResult {
        let model_name = model.clone().unwrap_or_else(|| "unknown".to_string());
        let tool_count = tools.as_ref().map_or(0, Vec::len);

        let msg = format!("🚀 Starting with **{model_name}** | {tool_count} tools available");

        ParseResult::with_activity(ParsedActivity::thought(msg))
    }

    /// Parse an Assistant event
    fn parse_assistant_event(&mut self, message: &Option<AssistantMessage>) -> ParseResult {
        let mut result = ParseResult::empty();

        let Some(msg) = message else {
            return result;
        };

        let Some(ref content) = msg.content else {
            return result;
        };

        for block in content {
            match block {
                ContentBlock::ToolUse { name, input, .. } => {
                    // Format input summary
                    let input_summary = input
                        .as_ref()
                        .map_or_else(String::new, |v| truncate_chars(&v.to_string(), 150));

                    // Store state for pairing with result
                    self.tool_state.current_tool = Some(name.clone());
                    self.tool_state.current_input = Some(input_summary.clone());
                    self.tool_state.current_input_json.clone_from(input);

                    result
                        .activities
                        .push(ParsedActivity::action(name, &input_summary));
                }
                ContentBlock::Text { text } => {
                    // Emit significant text as thought
                    // Skip boilerplate phrases
                    if text.chars().count() > 50
                        && !text.starts_with("I'll")
                        && !text.starts_with("Let me")
                        && !text.starts_with("Now I")
                    {
                        let display_text = truncate_chars(text, 500);
                        result
                            .activities
                            .push(ParsedActivity::thought(display_text));
                    }
                }
                ContentBlock::ToolResult { .. } => {
                    // Tool results are typically in User events
                }
            }
        }

        result
    }

    /// Parse a User event (tool result)
    fn parse_user_event(&mut self, tool_use_result: &Option<String>) -> ParseResult {
        let mut result = ParseResult::empty();

        // Get tool info from state
        let tool_name = self
            .tool_state
            .current_tool
            .take()
            .unwrap_or_else(|| "Tool".to_string());
        let tool_input = self.tool_state.current_input.take().unwrap_or_default();
        let tool_input_json = self.tool_state.current_input_json.take();

        // Format result
        let is_error = tool_use_result
            .as_ref()
            .is_some_and(|r| r.contains("error") || r.contains("Error"));

        let status_emoji = if is_error { "❌" } else { "✅" };
        let result_preview = tool_use_result
            .as_ref()
            .map_or_else(|| "No result".to_string(), |r| truncate_chars(r, 200));

        let formatted_result = format!("{status_emoji} {result_preview}");

        result.activities.push(ParsedActivity::action_complete(
            &tool_name,
            &tool_input,
            formatted_result,
        ));

        // Track file operations for artifact trail
        if !is_error {
            if let Some(ref input_json) = tool_input_json {
                let tool_lower = tool_name.to_lowercase();

                // Track file operations based on tool name
                if tool_lower.contains("write_file") || tool_lower.contains("create") {
                    if let Some(path) = extract_path_from_input(input_json) {
                        result.add_artifact(ArtifactOperation::Create { path });
                    }
                } else if tool_lower.contains("read") || tool_lower.contains("file") {
                    if let Some(path) = extract_path_from_input(input_json) {
                        result.add_artifact(ArtifactOperation::Read { path });
                    }
                } else if tool_lower.contains("edit") || tool_lower.contains("modify") {
                    if let Some(path) = extract_path_from_input(input_json) {
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

    /// Parse a Result event (session completion)
    fn parse_result_event(
        &mut self,
        duration_ms: Option<u64>,
        total_cost_usd: Option<f64>,
        num_turns: Option<u32>,
        subtype: &Option<String>,
    ) -> ParseResult {
        // Update stats
        if let Some(ms) = duration_ms {
            self.stats.set_duration(ms);
        }
        if let Some(cost) = total_cost_usd {
            self.stats.add_cost(cost);
        }
        if let Some(turns) = num_turns {
            self.stats.turn_count = turns;
        }

        #[allow(clippy::cast_precision_loss)]
        let duration_secs = duration_ms.map_or(0.0, |ms| ms as f64 / 1000.0);
        let turns = num_turns.unwrap_or(0);
        let is_error = subtype.as_deref() == Some("error");

        let summary = format!(
            "**Completed** | {duration_secs:.1}s | ${:.4} | {turns} turns",
            self.stats.total_cost
        );

        let activity = if is_error {
            ParsedActivity::error(summary)
        } else {
            ParsedActivity::response(summary)
        };

        ParseResult::with_activity(activity)
    }
}

impl Default for ClaudeParser {
    fn default() -> Self {
        Self::new()
    }
}

impl StreamParser for ClaudeParser {
    fn id(&self) -> &'static str {
        "claude"
    }

    fn can_parse(&self, sample_line: &str) -> bool {
        // Claude stream-json has "type" field with "system", "assistant", "user", or "result"
        sample_line.contains("\"type\":\"system\"")
            || sample_line.contains("\"type\":\"assistant\"")
            || sample_line.contains("\"type\":\"user\"")
            || sample_line.contains("\"type\":\"result\"")
    }

    fn parse_init_from_log(&self, log_content: &str) -> Option<InitInfo> {
        // Claude uses System event in stream for init, not log file
        // But also check log file for MCP diagnostics
        init_log::parse_mcp_init_from_log(log_content)
    }

    fn parse_line(&mut self, line: &str) -> ParseResult {
        let Ok(event) = serde_json::from_str::<ClaudeStreamEvent>(line) else {
            return ParseResult::empty();
        };

        match event {
            ClaudeStreamEvent::System { model, tools, .. } => {
                self.parse_system_event(&model, &tools)
            }
            ClaudeStreamEvent::Assistant { message, .. } => self.parse_assistant_event(&message),
            ClaudeStreamEvent::User {
                tool_use_result, ..
            } => self.parse_user_event(&tool_use_result),
            ClaudeStreamEvent::Result {
                duration_ms,
                total_cost_usd,
                num_turns,
                subtype,
                ..
            } => {
                self.stats.increment_turns();
                self.parse_result_event(duration_ms, total_cost_usd, num_turns, &subtype)
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
    // Try common field names for file paths
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
        let parser = ClaudeParser::new();
        assert!(parser.can_parse(r#"{"type":"system","model":"claude-opus-4"}"#));
        assert!(parser.can_parse(r#"{"type":"assistant","message":{}}"#));
        assert!(parser.can_parse(r#"{"type":"result","duration_ms":5000}"#));
        assert!(!parser.can_parse(r#"{"tool_name":"read_file"}"#));
    }

    #[test]
    fn test_parse_system_event() {
        let mut parser = ClaudeParser::new();
        let line =
            r#"{"type":"system","model":"claude-opus-4","tools":["read_file","write_file"]}"#;

        let result = parser.parse_line(line);
        assert_eq!(result.activities.len(), 1);

        match &result.activities[0] {
            ParsedActivity::Thought { body, .. } => {
                assert!(body.contains("claude-opus-4"));
                assert!(body.contains("2 tools"));
            }
            _ => panic!("Expected thought activity"),
        }
    }

    #[test]
    fn test_parse_tool_use() {
        let mut parser = ClaudeParser::new();

        // Tool use event
        let tool_use = r#"{"type":"assistant","message":{"content":[{"type":"tool_use","name":"read_file","input":{"path":"src/lib.rs"}}]}}"#;
        let result = parser.parse_line(tool_use);
        assert_eq!(result.activities.len(), 1);

        match &result.activities[0] {
            ParsedActivity::Action { name, .. } => {
                assert_eq!(name, "read_file");
            }
            _ => panic!("Expected action activity"),
        }

        // Tool result event
        let tool_result = r#"{"type":"user","tool_use_result":"file contents here"}"#;
        let result = parser.parse_line(tool_result);
        assert_eq!(result.activities.len(), 1);

        match &result.activities[0] {
            ParsedActivity::Action { name, result, .. } => {
                assert_eq!(name, "read_file");
                assert!(result.is_some());
            }
            _ => panic!("Expected action with result"),
        }
    }

    #[test]
    fn test_parse_result_event() {
        let mut parser = ClaudeParser::new();
        let line = r#"{"type":"result","duration_ms":5000,"total_cost_usd":0.05,"num_turns":3}"#;

        let result = parser.parse_line(line);
        assert_eq!(result.activities.len(), 1);

        match &result.activities[0] {
            ParsedActivity::Response { body } => {
                assert!(body.contains("Completed"));
                assert!(body.contains("5.0s"));
            }
            _ => panic!("Expected response activity"),
        }

        let stats = parser.get_stats();
        assert_eq!(stats.duration_ms, 5000);
        assert!((stats.total_cost - 0.05).abs() < f64::EPSILON);
    }

    #[test]
    fn test_truncate_chars() {
        assert_eq!(truncate_chars("hello", 10), "hello");
        assert_eq!(truncate_chars("hello world", 5), "hello...");
    }

    #[test]
    fn test_extract_path() {
        let input = serde_json::json!({"path": "src/main.rs"});
        assert_eq!(
            extract_path_from_input(&input),
            Some("src/main.rs".to_string())
        );

        let input = serde_json::json!({"file": "test.txt"});
        assert_eq!(
            extract_path_from_input(&input),
            Some("test.txt".to_string())
        );

        let input = serde_json::json!({"query": "SELECT *"});
        assert_eq!(extract_path_from_input(&input), None);
    }
}
