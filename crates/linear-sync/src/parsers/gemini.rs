//! Gemini CLI stream parser.
//!
//! Parses Gemini's JSONL output format similar to Codex.

use serde::Deserialize;
use serde_json::Value;

use super::{
    init_log, ArtifactOperation, InitInfo, ParseResult, ParsedActivity, StreamParser, StreamStats,
};

/// Gemini output structure
#[derive(Debug, Clone, Deserialize)]
pub struct GeminiOutput {
    /// Commands array (Gemini uses same format as Codex)
    #[serde(default)]
    pub commands: Vec<GeminiCommand>,
    /// Model name
    pub model: Option<String>,
    /// Usage statistics
    pub usage: Option<UsageStats>,
    /// Result text
    pub result: Option<String>,
    /// Error flag
    #[serde(default)]
    pub is_error: bool,
    /// Text output
    pub text: Option<String>,
}

/// A command in Gemini output
#[derive(Debug, Clone, Deserialize)]
pub struct GeminiCommand {
    /// Command/tool name
    #[serde(alias = "tool")]
    pub command: Option<String>,
    /// Arguments
    pub args: Option<Value>,
}

/// Usage statistics
#[derive(Debug, Clone, Deserialize)]
pub struct UsageStats {
    /// Input tokens
    #[serde(alias = "inputTokens")]
    pub input_tokens: Option<u64>,
    /// Output tokens
    #[serde(alias = "outputTokens")]
    pub output_tokens: Option<u64>,
}

/// Gemini stream parser implementation
pub struct GeminiParser {
    stats: StreamStats,
}

impl GeminiParser {
    /// Create a new Gemini parser
    #[must_use]
    pub fn new() -> Self {
        Self {
            stats: StreamStats::new(),
        }
    }
}

impl Default for GeminiParser {
    fn default() -> Self {
        Self::new()
    }
}

impl StreamParser for GeminiParser {
    fn id(&self) -> &'static str {
        "gemini"
    }

    fn can_parse(&self, sample_line: &str) -> bool {
        // Gemini outputs are similar to Codex but may have different markers
        (sample_line.contains("\"commands\"") || sample_line.contains("\"gemini\""))
            && !sample_line.contains("\"type\":\"system\"")
    }

    fn parse_init_from_log(&self, log_content: &str) -> Option<InitInfo> {
        init_log::parse_mcp_init_from_log(log_content)
    }

    fn parse_line(&mut self, line: &str) -> ParseResult {
        let Ok(output) = serde_json::from_str::<GeminiOutput>(line) else {
            return ParseResult::empty();
        };

        let mut activities = Vec::new();
        let mut artifacts = Vec::new();

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
                .map_or_else(String::new, |v| truncate_chars(&v.to_string(), 150));

            activities.push(ParsedActivity::action(&name, &input_summary));

            // Track file artifacts
            if let Some(ref args) = cmd.args {
                let name_lower = name.to_lowercase();
                if let Some(path) = extract_path(args) {
                    if name_lower.contains("write") || name_lower.contains("create") {
                        artifacts.push(ArtifactOperation::Create { path });
                    } else if name_lower.contains("read") {
                        artifacts.push(ArtifactOperation::Read { path });
                    }
                }
            }
        }

        // Handle text output as thought
        if let Some(text) = &output.text {
            if text.chars().count() > 30 {
                activities.push(ParsedActivity::thought(truncate_chars(text, 500)));
            }
        }

        // Handle result
        if let Some(result) = &output.result {
            self.stats.increment_turns();
            let summary = truncate_chars(result, 200);
            if output.is_error {
                activities.push(ParsedActivity::error(summary));
            } else {
                activities.push(ParsedActivity::response(format!(
                    "**Completed**: {summary}"
                )));
            }
        }

        ParseResult {
            activities,
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

fn truncate_chars(s: &str, max: usize) -> String {
    let chars: Vec<char> = s.chars().collect();
    if chars.len() <= max {
        s.to_string()
    } else {
        format!("{}...", chars[..max].iter().collect::<String>())
    }
}

fn extract_path(input: &Value) -> Option<String> {
    for field in &["path", "file_path", "file"] {
        if let Some(Value::String(p)) = input.get(field) {
            return Some(p.clone());
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_gemini_output() {
        let mut parser = GeminiParser::new();
        let line = r#"{"commands":[{"command":"read_file","args":{"path":"test.rs"}}]}"#;
        let result = parser.parse_line(line);
        assert_eq!(result.activities.len(), 1);
    }
}
