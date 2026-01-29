//! Codex CLI stream parser.
//!
//! Parses Codex's JSON output format which contains:
//! - `commands`: Array of command objects with `command` and `args`
//! - `model`: Model name used
//! - `usage`: Token usage statistics
//!
//! # Example Output
//!
//! ```json
//! {
//!   "commands": [
//!     {"command": "local_shell", "args": {"command": "ls -la"}}
//!   ],
//!   "model": "gpt-5-codex",
//!   "usage": {"input_tokens": 100, "output_tokens": 200}
//! }
//! ```

use serde::Deserialize;
use serde_json::Value;

use super::{
    init_log, ArtifactOperation, InitInfo, ParseResult, ParsedActivity, StreamParser, StreamStats,
};

/// Codex JSON output structure
#[derive(Debug, Clone, Deserialize)]
pub struct CodexOutput {
    /// Commands to execute
    #[serde(default)]
    pub commands: Vec<CodexCommand>,
    /// Model name
    pub model: Option<String>,
    /// Token usage
    pub usage: Option<UsageStats>,
    /// Result text
    pub result: Option<String>,
    /// Error flag
    #[serde(default)]
    pub is_error: bool,
}

/// A command in Codex output
#[derive(Debug, Clone, Deserialize)]
pub struct CodexCommand {
    /// Command name
    pub command: Option<String>,
    /// Command arguments
    pub args: Option<Value>,
}

/// Usage statistics
#[derive(Debug, Clone, Deserialize)]
pub struct UsageStats {
    /// Input tokens
    pub input_tokens: Option<u64>,
    /// Output tokens
    pub output_tokens: Option<u64>,
}

/// Codex stream parser implementation
pub struct CodexParser {
    stats: StreamStats,
    cli_id: &'static str,
}

impl CodexParser {
    /// Create a new Codex parser
    #[must_use]
    pub fn new() -> Self {
        Self {
            stats: StreamStats::new(),
            cli_id: "codex",
        }
    }

    /// Create a parser for the Code CLI (same format as Codex)
    #[must_use]
    pub fn code() -> Self {
        Self {
            stats: StreamStats::new(),
            cli_id: "code",
        }
    }

    /// Parse commands into activities
    fn parse_commands(&mut self, commands: &[CodexCommand]) -> Vec<ParsedActivity> {
        let mut activities = Vec::new();

        for (idx, cmd) in commands.iter().enumerate() {
            let name = cmd
                .command
                .clone()
                .unwrap_or_else(|| format!("command_{idx}"));

            let input_summary = cmd
                .args
                .as_ref()
                .map_or_else(String::new, |v| truncate_chars(&v.to_string(), 150));

            activities.push(ParsedActivity::action(&name, &input_summary));
        }

        activities
    }

    /// Extract artifact operations from commands
    fn extract_artifacts(&self, commands: &[CodexCommand]) -> Vec<ArtifactOperation> {
        let mut artifacts = Vec::new();

        for cmd in commands {
            let name_lower = cmd
                .command
                .as_ref()
                .map(|s| s.to_lowercase())
                .unwrap_or_default();

            if let Some(ref args) = cmd.args {
                if let Some(path) = extract_path_from_input(args) {
                    if name_lower.contains("write") || name_lower.contains("create") {
                        artifacts.push(ArtifactOperation::Create { path });
                    } else if name_lower.contains("read") {
                        artifacts.push(ArtifactOperation::Read { path });
                    } else if name_lower.contains("edit") || name_lower.contains("modify") {
                        artifacts.push(ArtifactOperation::Modify {
                            path,
                            summary: "Modified".to_string(),
                        });
                    }
                }
            }
        }

        artifacts
    }
}

impl Default for CodexParser {
    fn default() -> Self {
        Self::new()
    }
}

impl StreamParser for CodexParser {
    fn id(&self) -> &'static str {
        self.cli_id
    }

    fn can_parse(&self, sample_line: &str) -> bool {
        // Codex output has "commands" array
        sample_line.contains("\"commands\"") && sample_line.contains("\"command\"")
    }

    fn parse_init_from_log(&self, log_content: &str) -> Option<InitInfo> {
        init_log::parse_mcp_init_from_log(log_content)
    }

    fn parse_line(&mut self, line: &str) -> ParseResult {
        let Ok(output) = serde_json::from_str::<CodexOutput>(line) else {
            return ParseResult::empty();
        };

        // Update stats from usage
        if let Some(usage) = &output.usage {
            if let (Some(input), Some(output_tokens)) = (usage.input_tokens, usage.output_tokens) {
                self.stats.add_tokens(input, output_tokens);
            }
        }

        // Parse commands into activities
        let activities = self.parse_commands(&output.commands);
        let artifacts = self.extract_artifacts(&output.commands);

        // Add result/completion activity if present
        let mut all_activities = activities;
        if let Some(result) = &output.result {
            self.stats.increment_turns();
            let summary = truncate_chars(result, 200);
            if output.is_error {
                all_activities.push(ParsedActivity::error(summary));
            } else {
                all_activities.push(ParsedActivity::response(format!(
                    "**Completed**: {summary}"
                )));
            }
        }

        ParseResult {
            activities: all_activities,
            artifact_ops: artifacts,
        }
    }

    fn get_stats(&self) -> StreamStats {
        self.stats.clone()
    }

    fn reset(&mut self) {
        self.stats = StreamStats::new();
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
        let parser = CodexParser::new();
        assert!(parser.can_parse(r#"{"commands":[{"command":"ls","args":{}}]}"#));
        assert!(!parser.can_parse(r#"{"type":"assistant"}"#));
    }

    #[test]
    fn test_parse_commands() {
        let mut parser = CodexParser::new();

        let line = r#"{"commands":[{"command":"local_shell","args":{"command":"ls -la"}}],"model":"gpt-5"}"#;
        let result = parser.parse_line(line);
        assert_eq!(result.activities.len(), 1);

        match &result.activities[0] {
            ParsedActivity::Action { name, .. } => {
                assert_eq!(name, "local_shell");
            }
            _ => panic!("Expected action activity"),
        }
    }

    #[test]
    fn test_code_parser_id() {
        let codex = CodexParser::new();
        let code = CodexParser::code();

        assert_eq!(codex.id(), "codex");
        assert_eq!(code.id(), "code");
    }

    #[test]
    fn test_parse_with_result() {
        let mut parser = CodexParser::new();

        let line = r#"{"commands":[],"result":"Task completed","is_error":false,"usage":{"input_tokens":100,"output_tokens":200}}"#;
        let result = parser.parse_line(line);
        assert_eq!(result.activities.len(), 1);

        match &result.activities[0] {
            ParsedActivity::Response { body } => {
                assert!(body.contains("Completed"));
            }
            _ => panic!("Expected response activity"),
        }

        let stats = parser.get_stats();
        assert_eq!(stats.input_tokens, Some(100));
        assert_eq!(stats.output_tokens, Some(200));
    }
}
