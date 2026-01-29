//! Test utilities for the linear-sink crate.
//!
//! Provides utilities for testing CLI stream parsing and Linear activity emission.

use std::collections::HashMap;
use std::process::Command;

/// CLI availability information for E2E tests.
#[derive(Debug, Clone)]
pub struct CliAvailability {
    /// CLI name
    pub name: String,
    /// Whether the CLI binary is found in PATH
    pub installed: bool,
    /// Version string if available
    pub version: Option<String>,
    /// Path to the executable
    pub path: Option<String>,
    /// Error message if check failed
    pub error: Option<String>,
}

impl CliAvailability {
    /// Check if a CLI is available by running `which` and version check.
    #[must_use]
    pub fn check(cli_name: &str) -> Self {
        let path = Command::new("which")
            .arg(cli_name)
            .output()
            .ok()
            .and_then(|o| {
                if o.status.success() {
                    String::from_utf8(o.stdout)
                        .ok()
                        .map(|s| s.trim().to_string())
                } else {
                    None
                }
            });

        let installed = path.is_some();

        let (version, error) = if installed {
            let version_output = Command::new(cli_name)
                .arg("--version")
                .output()
                .ok()
                .and_then(|o| {
                    if o.status.success() {
                        String::from_utf8(o.stdout)
                            .ok()
                            .map(|s| s.lines().next().unwrap_or("").trim().to_string())
                    } else {
                        String::from_utf8(o.stderr)
                            .ok()
                            .map(|s| s.lines().next().unwrap_or("").trim().to_string())
                    }
                });
            (version_output, None)
        } else {
            (None, Some(format!("{cli_name} not found in PATH")))
        };

        Self {
            name: cli_name.to_string(),
            installed,
            version,
            path,
            error,
        }
    }
}

/// All supported CLI names for E2E testing.
pub const SUPPORTED_CLIS: &[&str] = &[
    "claude", "cursor", "codex", "factory", "gemini", "opencode", "code", "dexter",
];

/// Check availability of all supported CLIs.
#[must_use]
pub fn check_all_clis() -> HashMap<String, CliAvailability> {
    SUPPORTED_CLIS
        .iter()
        .map(|cli| ((*cli).to_string(), CliAvailability::check(cli)))
        .collect()
}

/// Sample stream output for testing parsers without running actual CLIs.
/// These are based on real CLI output formats.
pub mod sample_streams {
    /// Claude stream-json sample with system event.
    pub const CLAUDE_INIT: &str = r#"{"type":"system","subtype":"init","session_id":"test-session","tools":["Read","Write","Shell"],"model":"claude-sonnet-4-20250514","cwd":"/workspace"}"#;

    /// Claude assistant message sample.
    pub const CLAUDE_ASSISTANT: &str = r#"{"type":"assistant","message":{"id":"msg_1","content":[{"type":"text","text":"I'll help you with that task."}]}}"#;

    /// Claude tool use sample.
    pub const CLAUDE_TOOL_USE: &str = r#"{"type":"assistant","message":{"id":"msg_2","content":[{"type":"tool_use","id":"tool_1","name":"Read","input":{"path":"/workspace/README.md"}}]}}"#;

    /// Claude result sample.
    pub const CLAUDE_RESULT: &str = r#"{"type":"result","subtype":"success","session_id":"test-session","cost_usd":0.0025,"duration_ms":1500,"num_turns":3}"#;

    /// Factory message sample.
    pub const FACTORY_MESSAGE: &str = r#"{"type":"message","content":"Analyzing the codebase..."}"#;

    /// Factory tool call sample.
    pub const FACTORY_TOOL_CALL: &str = r#"{"type":"tool_call","name":"read_file","input":{"path":"src/main.rs"},"call_id":"call_1"}"#;

    /// Factory result sample.
    pub const FACTORY_RESULT: &str = r#"{"type":"result","success":true,"output":"Task completed","usage":{"input_tokens":100,"output_tokens":50}}"#;

    /// Codex/Code output sample.
    pub const CODEX_OUTPUT: &str = r#"{"model":"gpt-4","commands":[{"type":"shell","command":"ls -la"},{"type":"write","path":"test.txt","content":"hello"}],"usage":{"input":50,"output":25},"result":"completed"}"#;

    /// Gemini output sample.
    pub const GEMINI_OUTPUT: &str = r#"{"model":"gemini-1.5-pro","commands":[{"type":"read","path":"file.rs"}],"text":"Processing request...","usage":{"input":30,"output":20},"result":"success"}"#;

    /// `OpenCode` output sample.
    pub const OPENCODE_OUTPUT: &str = r#"{"model":"gpt-4-turbo","commands":[{"type":"edit","path":"main.rs","content":"fn main() {}"}],"text":"Applied changes","usage":{"input":40,"output":30},"result":"done"}"#;

    /// Dexter output sample.
    pub const DEXTER_OUTPUT: &str = r#"{"model":"dexter-v1","actions":[{"type":"analyze","target":"portfolio.json"},{"type":"report","format":"markdown"}],"result":"Analysis complete","is_error":false}"#;

    /// Cursor output sample (commands-based).
    pub const CURSOR_OUTPUT: &str = r#"{"commands":[{"type":"edit","file":"app.tsx","changes":"added component"}],"model":"gpt-4","result":"success"}"#;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cli_availability_check() {
        // This will check whatever CLIs are actually installed
        let availability = CliAvailability::check("ls"); // ls should always exist
        assert!(availability.installed);
        assert!(availability.path.is_some());
    }

    #[test]
    fn test_check_all_clis() {
        let all = check_all_clis();
        assert_eq!(all.len(), SUPPORTED_CLIS.len());

        // Verify we can check availability for all CLIs
        for cli in SUPPORTED_CLIS {
            assert!(
                all.contains_key(*cli),
                "Should have availability info for {cli}"
            );
        }
    }

    #[test]
    fn test_sample_streams_are_valid_json() {
        use sample_streams::*;

        let samples = [
            ("claude_init", CLAUDE_INIT),
            ("claude_assistant", CLAUDE_ASSISTANT),
            ("claude_tool_use", CLAUDE_TOOL_USE),
            ("claude_result", CLAUDE_RESULT),
            ("factory_message", FACTORY_MESSAGE),
            ("factory_tool_call", FACTORY_TOOL_CALL),
            ("factory_result", FACTORY_RESULT),
            ("codex_output", CODEX_OUTPUT),
            ("gemini_output", GEMINI_OUTPUT),
            ("opencode_output", OPENCODE_OUTPUT),
            ("dexter_output", DEXTER_OUTPUT),
            ("cursor_output", CURSOR_OUTPUT),
        ];

        for (name, sample) in samples {
            let parsed: Result<serde_json::Value, _> = serde_json::from_str(sample);
            assert!(
                parsed.is_ok(),
                "Sample {name} should be valid JSON: {sample}"
            );
        }
    }
}
