//! Cursor CLI stream parser.
//!
//! Parses Cursor's JSON output format.
//!
//! Note: Cursor is a skeleton implementation - the exact output format
//! needs further investigation during integration testing.

use serde::Deserialize;
use serde_json::Value;

use super::{init_log, InitInfo, ParseResult, ParsedActivity, StreamParser, StreamStats};

/// Cursor output structure (placeholder)
#[derive(Debug, Clone, Deserialize)]
pub struct CursorOutput {
    /// Commands/actions
    #[serde(default)]
    pub commands: Vec<CursorCommand>,
    /// Text content
    pub text: Option<String>,
    /// Result
    pub result: Option<String>,
    /// Model name
    pub model: Option<String>,
    /// Error flag
    #[serde(default)]
    pub is_error: bool,
}

/// A command in Cursor output
#[derive(Debug, Clone, Deserialize)]
pub struct CursorCommand {
    /// Command name
    pub command: Option<String>,
    /// Tool name (alternative)
    pub tool: Option<String>,
    /// Arguments
    pub args: Option<Value>,
    /// Input (alternative)
    pub input: Option<Value>,
}

/// Cursor stream parser
pub struct CursorParser {
    stats: StreamStats,
}

impl CursorParser {
    /// Create a new Cursor parser
    #[must_use]
    pub fn new() -> Self {
        Self {
            stats: StreamStats::new(),
        }
    }
}

impl Default for CursorParser {
    fn default() -> Self {
        Self::new()
    }
}

impl StreamParser for CursorParser {
    fn id(&self) -> &'static str {
        "cursor"
    }

    fn can_parse(&self, sample_line: &str) -> bool {
        // Cursor format detection - may need refinement
        sample_line.contains("\"cursor\"") || sample_line.contains("\"editor\"")
    }

    fn parse_init_from_log(&self, log_content: &str) -> Option<InitInfo> {
        init_log::parse_mcp_init_from_log(log_content)
    }

    fn parse_line(&mut self, line: &str) -> ParseResult {
        let Ok(output) = serde_json::from_str::<CursorOutput>(line) else {
            return ParseResult::empty();
        };

        let mut activities = Vec::new();

        // Parse commands
        for cmd in &output.commands {
            let name = cmd
                .command
                .clone()
                .or_else(|| cmd.tool.clone())
                .unwrap_or_else(|| "command".to_string());

            let input_summary = cmd
                .args
                .as_ref()
                .or(cmd.input.as_ref())
                .map_or_else(String::new, |v| truncate(&v.to_string(), 150));

            activities.push(ParsedActivity::action(&name, &input_summary));
        }

        // Handle text
        if let Some(text) = &output.text {
            if text.chars().count() > 30 {
                activities.push(ParsedActivity::thought(truncate(text, 500)));
            }
        }

        // Handle result
        if let Some(result) = &output.result {
            self.stats.increment_turns();
            let summary = truncate(result, 200);
            if output.is_error {
                activities.push(ParsedActivity::error(summary));
            } else {
                activities.push(ParsedActivity::response(format!(
                    "**Completed**: {summary}"
                )));
            }
        }

        ParseResult::with_activities(activities)
    }

    fn get_stats(&self) -> StreamStats {
        self.stats.clone()
    }

    fn reset(&mut self) {
        self.stats = StreamStats::new();
    }
}

fn truncate(s: &str, max: usize) -> String {
    let chars: Vec<char> = s.chars().collect();
    if chars.len() <= max {
        s.to_string()
    } else {
        format!("{}...", chars[..max].iter().collect::<String>())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_cursor_output() {
        let mut parser = CursorParser::new();
        let line = r#"{"commands":[{"command":"edit","args":{"file":"test.rs"}}]}"#;
        let result = parser.parse_line(line);
        assert_eq!(result.activities.len(), 1);
    }
}
