//! CLI-specific stream parsers for Linear activity emission.
//!
//! This module provides the `StreamParser` trait and implementations
//! for all supported CLIs:
//!
//! - **Claude**: `stream-json` format with System event containing tools
//! - **Factory**: `stream-json` format with `tool_call` events
//! - **Codex/Code**: JSONL with commands array
//! - **Gemini**: JSONL format with commands array
//! - **`OpenCode`**: JSONL format with commands array
//! - **Dexter**: Single JSON format with actions array
//! - **Cursor**: JSON format (skeleton implementation)
//!
//! # Parser Selection
//!
//! Parsers are selected via the `CLI_TYPE` environment variable or
//! auto-detected from stream content using `ParserRegistry::detect_parser`.

pub mod claude;
pub mod codex;
pub mod cursor;
pub mod dexter;
pub mod factory;
pub mod gemini;
pub mod init_log;
pub mod opencode;
pub mod registry;

// Re-export parsers for convenience
pub use claude::ClaudeParser;
pub use codex::CodexParser;
pub use cursor::CursorParser;
pub use dexter::DexterParser;
pub use factory::FactoryParser;
pub use gemini::GeminiParser;
pub use opencode::OpenCodeParser;
pub use registry::ParserRegistry;

/// Initialization information extracted from CLI startup.
///
/// This captures MCP tool configuration and model info that should
/// be displayed in the first section of the Linear agent dialog.
#[derive(Debug, Clone, Default)]
pub struct InitInfo {
    /// Model name if detected (e.g., "claude-opus-4", "gemini-2.0-flash")
    pub model: Option<String>,
    /// Number of tools available
    pub tool_count: usize,
    /// Tool names or prefixes
    pub tool_names: Vec<String>,
    /// MCP server names
    pub mcp_servers: Vec<String>,
}

impl InitInfo {
    /// Create a new empty `InitInfo`
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Create `InitInfo` with model and tools
    #[must_use]
    pub fn with_model_and_tools(model: impl Into<String>, tools: Vec<String>) -> Self {
        Self {
            model: Some(model.into()),
            tool_count: tools.len(),
            tool_names: tools,
            mcp_servers: Vec::new(),
        }
    }

    /// Format as a summary string for Linear activity
    #[must_use]
    pub fn to_summary(&self) -> String {
        let model_part = self
            .model
            .as_ref()
            .map_or_else(String::new, |m| format!("**{m}**"));

        let tools_part = if self.tool_count > 0 {
            format!("{} tools available", self.tool_count)
        } else {
            String::new()
        };

        match (model_part.is_empty(), tools_part.is_empty()) {
            (true, true) => "Starting agent...".to_string(),
            (true, false) => format!("Starting with {tools_part}"),
            (false, true) => format!("Starting with {model_part}"),
            (false, false) => format!("Starting with {model_part} | {tools_part}"),
        }
    }
}

/// Result of parsing a stream line.
///
/// Contains activities to emit and artifact operations for tracking.
#[derive(Debug, Clone, Default)]
pub struct ParseResult {
    /// Activities to emit to Linear
    pub activities: Vec<ParsedActivity>,
    /// Artifact operations for file tracking
    pub artifact_ops: Vec<ArtifactOperation>,
}

impl ParseResult {
    /// Create an empty result
    #[must_use]
    pub fn empty() -> Self {
        Self::default()
    }

    /// Create a result with a single activity
    #[must_use]
    pub fn with_activity(activity: ParsedActivity) -> Self {
        Self {
            activities: vec![activity],
            artifact_ops: Vec::new(),
        }
    }

    /// Create a result with multiple activities
    #[must_use]
    pub fn with_activities(activities: Vec<ParsedActivity>) -> Self {
        Self {
            activities,
            artifact_ops: Vec::new(),
        }
    }

    /// Add an artifact operation
    pub fn add_artifact(&mut self, op: ArtifactOperation) {
        self.artifact_ops.push(op);
    }

    /// Check if the result is empty
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.activities.is_empty() && self.artifact_ops.is_empty()
    }
}

/// A parsed activity ready for emission to Linear.
#[derive(Debug, Clone)]
pub enum ParsedActivity {
    /// A thought or internal note
    Thought {
        /// Thought body (markdown supported)
        body: String,
        /// Whether this is ephemeral (replaced by next activity)
        ephemeral: bool,
    },
    /// A tool invocation
    Action {
        /// Tool name
        name: String,
        /// Tool input/parameters (JSON or description)
        input: String,
        /// Tool result (if completed)
        result: Option<String>,
    },
    /// Final response
    Response {
        /// Response body (markdown supported)
        body: String,
    },
    /// Error report
    Error {
        /// Error body (markdown supported)
        body: String,
    },
}

impl ParsedActivity {
    /// Create an ephemeral thought
    #[must_use]
    pub fn ephemeral_thought(body: impl Into<String>) -> Self {
        Self::Thought {
            body: body.into(),
            ephemeral: true,
        }
    }

    /// Create a persistent thought
    #[must_use]
    pub fn thought(body: impl Into<String>) -> Self {
        Self::Thought {
            body: body.into(),
            ephemeral: false,
        }
    }

    /// Create an action (tool call in progress)
    #[must_use]
    pub fn action(name: impl Into<String>, input: impl Into<String>) -> Self {
        Self::Action {
            name: name.into(),
            input: input.into(),
            result: None,
        }
    }

    /// Create an action with result
    #[must_use]
    pub fn action_complete(
        name: impl Into<String>,
        input: impl Into<String>,
        result: impl Into<String>,
    ) -> Self {
        Self::Action {
            name: name.into(),
            input: input.into(),
            result: Some(result.into()),
        }
    }

    /// Create a response
    #[must_use]
    pub fn response(body: impl Into<String>) -> Self {
        Self::Response { body: body.into() }
    }

    /// Create an error
    #[must_use]
    pub fn error(body: impl Into<String>) -> Self {
        Self::Error { body: body.into() }
    }
}

/// An artifact operation for tracking file changes.
///
/// Used to maintain an "artifact trail" showing what files
/// the agent has touched during the session.
#[derive(Debug, Clone)]
pub enum ArtifactOperation {
    /// File was created
    Create {
        /// File path
        path: String,
    },
    /// File was modified
    Modify {
        /// File path
        path: String,
        /// Summary of changes (e.g., "Added 15 lines")
        summary: String,
    },
    /// File was read
    Read {
        /// File path
        path: String,
    },
    /// File was deleted
    Delete {
        /// File path
        path: String,
    },
}

impl ArtifactOperation {
    /// Get the path of the artifact
    #[must_use]
    pub fn path(&self) -> &str {
        match self {
            Self::Create { path }
            | Self::Modify { path, .. }
            | Self::Read { path }
            | Self::Delete { path } => path,
        }
    }
}

/// Accumulated statistics from stream parsing.
///
/// Tracks cost, duration, and token usage for session summary.
#[derive(Debug, Clone, Default)]
pub struct StreamStats {
    /// Total cost in USD
    pub total_cost: f64,
    /// Total duration in milliseconds
    pub duration_ms: u64,
    /// Number of turns/iterations
    pub turn_count: u32,
    /// Input tokens used
    pub input_tokens: Option<u64>,
    /// Output tokens generated
    pub output_tokens: Option<u64>,
}

impl StreamStats {
    /// Create empty stats
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Add cost
    pub fn add_cost(&mut self, cost: f64) {
        self.total_cost += cost;
    }

    /// Set duration
    pub fn set_duration(&mut self, ms: u64) {
        self.duration_ms = ms;
    }

    /// Increment turn count
    pub fn increment_turns(&mut self) {
        self.turn_count += 1;
    }

    /// Add token usage
    pub fn add_tokens(&mut self, input: u64, output: u64) {
        self.input_tokens = Some(self.input_tokens.unwrap_or(0) + input);
        self.output_tokens = Some(self.output_tokens.unwrap_or(0) + output);
    }

    /// Format as a summary string
    #[must_use]
    pub fn to_summary(&self) -> String {
        let mut parts = Vec::new();

        if self.duration_ms > 0 {
            #[allow(clippy::cast_precision_loss)]
            let secs = self.duration_ms as f64 / 1000.0;
            parts.push(format!("{secs:.1}s"));
        }

        if self.total_cost > 0.0 {
            parts.push(format!("${:.4}", self.total_cost));
        }

        if self.turn_count > 0 {
            parts.push(format!("{} turns", self.turn_count));
        }

        if let (Some(input), Some(output)) = (self.input_tokens, self.output_tokens) {
            parts.push(format!("{input}+{output} tokens"));
        }

        if parts.is_empty() {
            "Completed".to_string()
        } else {
            parts.join(" | ")
        }
    }
}

/// Trait for parsing CLI output streams into Linear activities.
///
/// Each CLI implementation parses its specific output format and
/// converts events into `ParsedActivity` items for emission.
///
/// # Example Implementation
///
/// ```rust,ignore
/// struct MyCliParser {
///     stats: StreamStats,
/// }
///
/// impl StreamParser for MyCliParser {
///     fn id(&self) -> &'static str { "my-cli" }
///
///     fn can_parse(&self, sample: &str) -> bool {
///         sample.contains("\"my_cli_marker\"")
///     }
///
///     fn parse_init_from_log(&self, log: &str) -> Option<InitInfo> {
///         // Parse MCP tool initialization from log file
///         init_log::parse_mcp_init_from_log(log)
///     }
///
///     fn parse_line(&mut self, line: &str) -> ParseResult {
///         // Parse JSON line and extract activities
///         ParseResult::empty()
///     }
///
///     fn get_stats(&self) -> StreamStats {
///         self.stats.clone()
///     }
///
///     fn reset(&mut self) {
///         self.stats = StreamStats::new();
///     }
/// }
/// ```
pub trait StreamParser: Send + Sync {
    /// Parser identifier (e.g., "claude", "factory", "codex")
    fn id(&self) -> &'static str;

    /// Check if this parser can handle the given sample line.
    ///
    /// Used for auto-detection when `CLI_TYPE` is not set.
    fn can_parse(&self, sample_line: &str) -> bool;

    /// Parse initialization info from log file (MCP tools display).
    ///
    /// Returns tool names/counts extracted from startup output.
    /// Most CLIs use shell script output from `mcp-check.sh.hbs`,
    /// while Claude has a structured System event.
    fn parse_init_from_log(&self, log_content: &str) -> Option<InitInfo>;

    /// Parse a single line from the stream file.
    ///
    /// Returns activities to emit and artifact operations to track.
    fn parse_line(&mut self, line: &str) -> ParseResult;

    /// Get accumulated stats for session summary.
    fn get_stats(&self) -> StreamStats;

    /// Reset parser state for a new session.
    fn reset(&mut self);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_init_info_summary() {
        let info = InitInfo::default();
        assert_eq!(info.to_summary(), "Starting agent...");

        let info = InitInfo {
            model: Some("claude-opus-4".to_string()),
            tool_count: 15,
            tool_names: vec!["read".to_string(), "write".to_string()],
            mcp_servers: vec![],
        };
        assert_eq!(
            info.to_summary(),
            "Starting with **claude-opus-4** | 15 tools available"
        );
    }

    #[test]
    fn test_parse_result_builders() {
        let result = ParseResult::empty();
        assert!(result.is_empty());

        let result = ParseResult::with_activity(ParsedActivity::thought("test"));
        assert_eq!(result.activities.len(), 1);
    }

    #[test]
    fn test_stream_stats_summary() {
        let mut stats = StreamStats::new();
        stats.duration_ms = 5000;
        stats.total_cost = 0.0123;
        stats.turn_count = 3;

        let summary = stats.to_summary();
        assert!(summary.contains("5.0s"));
        assert!(summary.contains("$0.0123"));
        assert!(summary.contains("3 turns"));
    }

    #[test]
    fn test_artifact_operation_path() {
        let op = ArtifactOperation::Create {
            path: "/test/file.rs".to_string(),
        };
        assert_eq!(op.path(), "/test/file.rs");

        let op = ArtifactOperation::Modify {
            path: "/test/mod.rs".to_string(),
            summary: "Added 10 lines".to_string(),
        };
        assert_eq!(op.path(), "/test/mod.rs");
    }
}
