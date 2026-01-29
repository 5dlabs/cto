//! Parser registry for CLI type selection and auto-detection.
//!
//! The registry manages all available parsers and provides:
//! - Selection by CLI type string
//! - Auto-detection from stream content
//! - Default parser fallback

use super::{
    claude::ClaudeParser, codex::CodexParser, cursor::CursorParser, dexter::DexterParser,
    factory::FactoryParser, gemini::GeminiParser, opencode::OpenCodeParser, StreamParser,
};

/// Parser registry for CLI-agnostic stream parsing.
///
/// # Example
///
/// ```rust,ignore
/// let registry = ParserRegistry::new();
///
/// // Select by CLI type
/// let parser = registry.get_by_cli_type("claude").unwrap();
///
/// // Or auto-detect from content
/// let sample = r#"{"type":"system","model":"claude-opus-4"}"#;
/// let parser = registry.detect_parser(sample).unwrap();
/// ```
pub struct ParserRegistry {
    parsers: Vec<Box<dyn StreamParser>>,
}

impl ParserRegistry {
    /// Create a new registry with all available parsers.
    #[must_use]
    pub fn new() -> Self {
        let parsers: Vec<Box<dyn StreamParser>> = vec![
            Box::new(ClaudeParser::new()),
            Box::new(FactoryParser::new()),
            Box::new(CodexParser::new()),
            Box::new(CodexParser::code()),
            Box::new(GeminiParser::new()),
            Box::new(OpenCodeParser::new()),
            Box::new(DexterParser::new()),
            Box::new(CursorParser::new()),
        ];

        Self { parsers }
    }

    /// Get a parser by CLI type identifier.
    ///
    /// # Arguments
    /// * `cli_type` - CLI identifier (e.g., "claude", "factory", "codex")
    ///
    /// # Returns
    /// * `Some(parser)` if a matching parser is found
    /// * `None` if no parser matches the CLI type
    #[must_use]
    pub fn get_by_cli_type(&self, cli_type: &str) -> Option<Box<dyn StreamParser>> {
        let cli_lower = cli_type.to_lowercase();
        match cli_lower.as_str() {
            "claude" => Some(Box::new(ClaudeParser::new())),
            "factory" => Some(Box::new(FactoryParser::new())),
            "codex" => Some(Box::new(CodexParser::new())),
            "code" => Some(Box::new(CodexParser::code())),
            "gemini" => Some(Box::new(GeminiParser::new())),
            "opencode" => Some(Box::new(OpenCodeParser::new())),
            "dexter" => Some(Box::new(DexterParser::new())),
            "cursor" => Some(Box::new(CursorParser::new())),
            _ => None,
        }
    }

    /// Auto-detect the appropriate parser from a sample line.
    ///
    /// Iterates through registered parsers and returns the first
    /// one that can handle the sample content.
    ///
    /// # Arguments
    /// * `sample` - A sample line from the stream file
    ///
    /// # Returns
    /// * `Some(parser)` if a matching parser is found
    /// * `None` if no parser can handle the content
    #[must_use]
    pub fn detect_parser(&self, sample: &str) -> Option<Box<dyn StreamParser>> {
        for parser in &self.parsers {
            if parser.can_parse(sample) {
                // Return a fresh instance based on detected type
                return self.get_by_cli_type(parser.id());
            }
        }
        None
    }

    /// Get all supported CLI types.
    #[must_use]
    pub fn supported_cli_types() -> Vec<&'static str> {
        vec![
            "claude", "factory", "codex", "code", "gemini", "opencode", "dexter", "cursor",
        ]
    }

    /// Create a parser from environment variable or auto-detect.
    ///
    /// Reads `CLI_TYPE` environment variable if set, otherwise
    /// attempts auto-detection from the sample content.
    #[must_use]
    pub fn from_env_or_detect(&self, sample: Option<&str>) -> Option<Box<dyn StreamParser>> {
        // Check CLI_TYPE env var first
        if let Ok(cli_type) = std::env::var("CLI_TYPE") {
            if !cli_type.is_empty() {
                return self.get_by_cli_type(&cli_type);
            }
        }

        // Fall back to auto-detection
        sample.and_then(|s| self.detect_parser(s))
    }
}

impl Default for ParserRegistry {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_by_cli_type() {
        let registry = ParserRegistry::new();

        assert!(registry.get_by_cli_type("claude").is_some());
        assert!(registry.get_by_cli_type("factory").is_some());
        assert!(registry.get_by_cli_type("codex").is_some());
        assert!(registry.get_by_cli_type("code").is_some());
        assert!(registry.get_by_cli_type("gemini").is_some());
        assert!(registry.get_by_cli_type("opencode").is_some());
        assert!(registry.get_by_cli_type("dexter").is_some());
        assert!(registry.get_by_cli_type("cursor").is_some());
        assert!(registry.get_by_cli_type("unknown").is_none());
    }

    #[test]
    fn test_detect_parser_claude() {
        let registry = ParserRegistry::new();
        let sample = r#"{"type":"system","model":"claude-opus-4"}"#;

        let parser = registry.detect_parser(sample).unwrap();
        assert_eq!(parser.id(), "claude");
    }

    #[test]
    fn test_detect_parser_factory() {
        let registry = ParserRegistry::new();
        let sample = r#"{"type":"tool_call","toolName":"Execute"}"#;

        let parser = registry.detect_parser(sample).unwrap();
        assert_eq!(parser.id(), "factory");
    }

    #[test]
    fn test_detect_parser_dexter() {
        let registry = ParserRegistry::new();
        let sample = r#"{"actions":[{"name":"search"}]}"#;

        let parser = registry.detect_parser(sample).unwrap();
        assert_eq!(parser.id(), "dexter");
    }

    #[test]
    fn test_supported_cli_types() {
        let types = ParserRegistry::supported_cli_types();
        assert!(types.contains(&"claude"));
        assert!(types.contains(&"factory"));
        assert_eq!(types.len(), 8);
    }
}
