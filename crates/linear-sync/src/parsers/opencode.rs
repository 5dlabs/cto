//! `OpenCode` CLI stream parser.
//!
//! Parses `OpenCode`'s JSONL output format.

use serde::Deserialize;
use serde_json::Value;

use super::{init_log, InitInfo, ParseResult, ParsedActivity, StreamParser, StreamStats};

/// `OpenCode` output structure
#[derive(Debug, Clone, Deserialize)]
pub struct OpenCodeOutput {
    /// Commands array
    #[serde(default)]
    pub commands: Vec<OpenCodeCommand>,
    /// Model name
    pub model: Option<String>,
    /// Usage statistics
    pub usage: Option<UsageStats>,
    /// Result text
    pub result: Option<String>,
    /// Text output
    pub text: Option<String>,
    /// Error flag
    #[serde(default)]
    pub is_error: bool,
}

/// A command in `OpenCode` output
#[derive(Debug, Clone, Deserialize)]
pub struct OpenCodeCommand {
    /// Command name
    pub command: Option<String>,
    /// Arguments
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

/// `OpenCode` stream parser
pub struct OpenCodeParser {
    stats: StreamStats,
}

impl OpenCodeParser {
    /// Create a new `OpenCode` parser
    #[must_use]
    pub fn new() -> Self {
        Self {
            stats: StreamStats::new(),
        }
    }
}

impl Default for OpenCodeParser {
    fn default() -> Self {
        Self::new()
    }
}

impl StreamParser for OpenCodeParser {
    fn id(&self) -> &'static str {
        "opencode"
    }

    fn can_parse(&self, sample_line: &str) -> bool {
        sample_line.contains("\"commands\"") && sample_line.contains("\"command\"")
    }

    fn parse_init_from_log(&self, log_content: &str) -> Option<InitInfo> {
        init_log::parse_mcp_init_from_log(log_content)
    }

    fn parse_line(&mut self, line: &str) -> ParseResult {
        let Ok(output) = serde_json::from_str::<OpenCodeOutput>(line) else {
            return ParseResult::empty();
        };

        let mut activities = Vec::new();

        // Update stats
        if let Some(usage) = &output.usage {
            if let (Some(input), Some(output_tokens)) = (usage.input_tokens, usage.output_tokens) {
                self.stats.add_tokens(input, output_tokens);
            }
        }

        // Parse commands
        for cmd in &output.commands {
            let name = cmd.command.clone().unwrap_or_else(|| "command".to_string());
            let input_summary = cmd
                .args
                .as_ref()
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
    fn test_parse_opencode_output() {
        let mut parser = OpenCodeParser::new();
        let line = r#"{"commands":[{"command":"shell","args":{"cmd":"ls"}}]}"#;
        let result = parser.parse_line(line);
        assert_eq!(result.activities.len(), 1);
    }
}
