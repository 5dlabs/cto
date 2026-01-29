//! Dexter CLI stream parser.
//!
//! Parses Dexter's single JSON output format with `actions` array.

use serde::Deserialize;
use serde_json::Value;

use super::{init_log, InitInfo, ParseResult, ParsedActivity, StreamParser, StreamStats};

/// Dexter output structure (single JSON, not JSONL)
#[derive(Debug, Clone, Deserialize)]
pub struct DexterOutput {
    /// Actions array
    #[serde(default)]
    pub actions: Vec<DexterAction>,
    /// Result text
    pub result: Option<String>,
    /// Model name
    pub model: Option<String>,
    /// Error flag
    #[serde(default)]
    pub is_error: bool,
}

/// A single action in Dexter output
#[derive(Debug, Clone, Deserialize)]
pub struct DexterAction {
    /// Action name/type
    #[serde(alias = "type")]
    pub name: Option<String>,
    /// Action input/parameters
    pub input: Option<Value>,
    /// Action output/result
    pub output: Option<String>,
}

/// Dexter stream parser
pub struct DexterParser {
    stats: StreamStats,
}

impl DexterParser {
    /// Create a new Dexter parser
    #[must_use]
    pub fn new() -> Self {
        Self {
            stats: StreamStats::new(),
        }
    }
}

impl Default for DexterParser {
    fn default() -> Self {
        Self::new()
    }
}

impl StreamParser for DexterParser {
    fn id(&self) -> &'static str {
        "dexter"
    }

    fn can_parse(&self, sample_line: &str) -> bool {
        // Dexter uses "actions" array instead of "commands"
        sample_line.contains("\"actions\"")
    }

    fn parse_init_from_log(&self, log_content: &str) -> Option<InitInfo> {
        // Dexter uses external API, no MCP config
        init_log::parse_mcp_init_from_log(log_content)
    }

    fn parse_line(&mut self, line: &str) -> ParseResult {
        let Ok(output) = serde_json::from_str::<DexterOutput>(line) else {
            return ParseResult::empty();
        };

        let mut activities = Vec::new();

        // Parse actions
        for action in &output.actions {
            let name = action.name.clone().unwrap_or_else(|| "action".to_string());
            let input_summary = action
                .input
                .as_ref()
                .map_or_else(String::new, |v| truncate(&v.to_string(), 150));

            if let Some(output) = &action.output {
                let result_summary = truncate(output, 200);
                activities.push(ParsedActivity::action_complete(
                    &name,
                    &input_summary,
                    result_summary,
                ));
            } else {
                activities.push(ParsedActivity::action(&name, &input_summary));
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
    fn test_parse_dexter_output() {
        let mut parser = DexterParser::new();
        let line = r#"{"actions":[{"name":"search","input":{"query":"test"},"output":"results"}]}"#;
        let result = parser.parse_line(line);
        assert_eq!(result.activities.len(), 1);
    }

    #[test]
    fn test_can_parse() {
        let parser = DexterParser::new();
        assert!(parser.can_parse(r#"{"actions":[]}"#));
        assert!(!parser.can_parse(r#"{"commands":[]}"#));
    }
}
