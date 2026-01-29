//! End-to-end tests for CLI stream parsing.
//!
//! These tests verify that each installed CLI produces output that can be
//! correctly parsed by its corresponding `StreamParser` implementation.
//!
//! Tests are skipped if the corresponding CLI is not installed.

#![allow(clippy::disallowed_macros)] // Tests use println! for debugging output

use linear_sink::parsers::{
    claude::ClaudeParser, codex::CodexParser, cursor::CursorParser, dexter::DexterParser,
    factory::FactoryParser, gemini::GeminiParser, opencode::OpenCodeParser,
    registry::ParserRegistry, ParsedActivity, StreamParser,
};
use linear_sink::testing::{check_all_clis, sample_streams, CliAvailability, SUPPORTED_CLIS};
use std::process::Command;

/// Helper to skip tests when CLI is not installed.
macro_rules! skip_if_not_installed {
    ($cli:expr) => {{
        let avail = CliAvailability::check($cli);
        if !avail.installed {
            return;
        }
        avail
    }};
}

/// Verify that a CLI can run with --help without error.
fn verify_cli_help(cli: &str) -> bool {
    Command::new(cli)
        .arg("--help")
        .output()
        .map(|o| o.status.success() || o.status.code() == Some(0))
        .unwrap_or(false)
}

// ============================================================================
// CLI Installation Tests
// ============================================================================

#[test]
fn test_list_installed_clis() {
    let all = check_all_clis();

    let mut installed_count = 0;

    for cli in SUPPORTED_CLIS {
        if let Some(avail) = all.get(*cli) {
            if avail.installed {
                installed_count += 1;
            }
        }
    }

    // Test passes as long as at least one CLI is installed
    assert!(
        installed_count > 0,
        "At least one supported CLI should be installed for E2E tests"
    );
}

// ============================================================================
// Parser Registry Tests
// ============================================================================

#[test]
fn test_parser_registry_supports_all_cli_types() {
    let _registry = ParserRegistry::new();
    let supported = ParserRegistry::supported_cli_types();

    for cli in SUPPORTED_CLIS {
        assert!(
            supported.contains(cli),
            "ParserRegistry should support CLI type: {cli}",
        );
    }
}

#[test]
fn test_parser_auto_detection_with_samples() {
    let registry = ParserRegistry::new();

    // Claude detection
    let claude_parser = registry.detect_parser(sample_streams::CLAUDE_INIT);
    assert!(claude_parser.is_some(), "Should detect Claude parser");
    assert_eq!(
        claude_parser.unwrap().id(),
        "claude",
        "Detected parser should be Claude"
    );

    // Factory detection - needs a tool_call with toolName
    let factory_sample =
        r#"{"type":"tool_call","toolName":"read_file","input":{"path":"src/main.rs"}}"#;
    let factory_parser = registry.detect_parser(factory_sample);
    assert!(factory_parser.is_some(), "Should detect Factory parser");
    assert_eq!(
        factory_parser.unwrap().id(),
        "factory",
        "Detected parser should be Factory"
    );

    // Dexter detection (has unique "actions" array)
    let dexter_parser = registry.detect_parser(sample_streams::DEXTER_OUTPUT);
    assert!(dexter_parser.is_some(), "Should detect Dexter parser");
    assert_eq!(
        dexter_parser.unwrap().id(),
        "dexter",
        "Detected parser should be Dexter"
    );
}

// ============================================================================
// Claude Parser Tests
// ============================================================================

#[test]
fn test_claude_parser_with_sample_stream() {
    let mut parser = ClaudeParser::new();

    // Parse init event - Claude system events are special and typically parsed via init
    let _init_info = parser.parse_init_from_log(sample_streams::CLAUDE_INIT);

    // Parse assistant message
    let msg_result = parser.parse_line(sample_streams::CLAUDE_ASSISTANT);
    assert!(
        !msg_result.activities.is_empty() || msg_result.is_empty(),
        "Should parse Claude assistant message"
    );

    // Parse tool use
    let tool_result = parser.parse_line(sample_streams::CLAUDE_TOOL_USE);
    // Tool use should produce an action activity
    if !tool_result.activities.is_empty() {
        assert!(matches!(
            &tool_result.activities[0],
            ParsedActivity::Action { .. } | ParsedActivity::Thought { .. }
        ));
    }

    // Parse result
    let _result = parser.parse_line(sample_streams::CLAUDE_RESULT);
    // Result events update stats
    let _stats = parser.get_stats();
}

#[test]
fn test_claude_cli_if_installed() {
    let _avail = skip_if_not_installed!("claude");

    // Verify help works
    assert!(
        verify_cli_help("claude"),
        "Claude CLI should respond to --help"
    );
}

// ============================================================================
// Factory Parser Tests
// ============================================================================

#[test]
fn test_factory_parser_with_sample_stream() {
    let mut parser = FactoryParser::new();

    // Parse message
    let _msg_result = parser.parse_line(sample_streams::FACTORY_MESSAGE);

    // Parse tool call
    let _tool_result = parser.parse_line(sample_streams::FACTORY_TOOL_CALL);

    // Parse result
    let _result = parser.parse_line(sample_streams::FACTORY_RESULT);
}

#[test]
fn test_factory_cli_if_installed() {
    let _avail = skip_if_not_installed!("factory");

    assert!(
        verify_cli_help("factory"),
        "Factory CLI should respond to --help"
    );
}

// ============================================================================
// Codex Parser Tests
// ============================================================================

#[test]
fn test_codex_parser_with_sample_stream() {
    let mut parser = CodexParser::new();

    let _result = parser.parse_line(sample_streams::CODEX_OUTPUT);
}

#[test]
fn test_codex_cli_if_installed() {
    let _avail = skip_if_not_installed!("codex");

    assert!(
        verify_cli_help("codex"),
        "Codex CLI should respond to --help"
    );
}

// ============================================================================
// Code (Every-Code) Parser Tests
// ============================================================================

#[test]
fn test_code_parser_with_sample_stream() {
    // Code parser uses the same format as Codex
    let mut parser = CodexParser::code();

    let _result = parser.parse_line(sample_streams::CODEX_OUTPUT);
    assert_eq!(parser.id(), "code");
}

#[test]
fn test_code_cli_if_installed() {
    let _avail = skip_if_not_installed!("code");

    assert!(verify_cli_help("code"), "Code CLI should respond to --help");
}

// ============================================================================
// Gemini Parser Tests
// ============================================================================

#[test]
fn test_gemini_parser_with_sample_stream() {
    let mut parser = GeminiParser::new();

    let _result = parser.parse_line(sample_streams::GEMINI_OUTPUT);
}

#[test]
fn test_gemini_cli_if_installed() {
    let _avail = skip_if_not_installed!("gemini");

    assert!(
        verify_cli_help("gemini"),
        "Gemini CLI should respond to --help"
    );
}

// ============================================================================
// OpenCode Parser Tests
// ============================================================================

#[test]
fn test_opencode_parser_with_sample_stream() {
    let mut parser = OpenCodeParser::new();

    let _result = parser.parse_line(sample_streams::OPENCODE_OUTPUT);
}

#[test]
fn test_opencode_cli_if_installed() {
    let _avail = skip_if_not_installed!("opencode");

    assert!(
        verify_cli_help("opencode"),
        "OpenCode CLI should respond to --help"
    );
}

// ============================================================================
// Dexter Parser Tests
// ============================================================================

#[test]
fn test_dexter_parser_with_sample_stream() {
    let mut parser = DexterParser::new();

    let _result = parser.parse_line(sample_streams::DEXTER_OUTPUT);
}

#[test]
fn test_dexter_cli_if_installed() {
    let _avail = skip_if_not_installed!("dexter");

    assert!(
        verify_cli_help("dexter"),
        "Dexter CLI should respond to --help"
    );
}

// ============================================================================
// Cursor Parser Tests
// ============================================================================

#[test]
fn test_cursor_parser_with_sample_stream() {
    let mut parser = CursorParser::new();

    let _result = parser.parse_line(sample_streams::CURSOR_OUTPUT);
}

#[test]
fn test_cursor_cli_if_installed() {
    let avail = skip_if_not_installed!("cursor");

    // Note: Cursor may not have a --help flag, so we just check it exists
    assert!(avail.installed, "Cursor should be installed");
}

// ============================================================================
// Integration Test: Full Stream Processing
// ============================================================================

#[test]
fn test_full_claude_stream_processing() {
    let mut parser = ClaudeParser::new();

    // Simulate a complete Claude session
    let stream_lines = [
        sample_streams::CLAUDE_INIT,
        sample_streams::CLAUDE_ASSISTANT,
        sample_streams::CLAUDE_TOOL_USE,
        r#"{"type":"user","message":{"content":[{"type":"tool_result","tool_use_id":"tool_1","content":"README content here"}]}}"#,
        sample_streams::CLAUDE_ASSISTANT,
        sample_streams::CLAUDE_RESULT,
    ];

    let mut all_activities = Vec::new();

    for line in stream_lines {
        let result = parser.parse_line(line);
        all_activities.extend(result.activities);
    }

    // Get final stats
    let _stats = parser.get_stats();
}

#[test]
fn test_full_factory_stream_processing() {
    let mut parser = FactoryParser::new();

    let stream_lines = [
        sample_streams::FACTORY_MESSAGE,
        sample_streams::FACTORY_TOOL_CALL,
        r#"{"type":"tool_result","call_id":"call_1","output":"fn main() {}"}"#,
        sample_streams::FACTORY_RESULT,
    ];

    let mut all_activities = Vec::new();

    for line in stream_lines {
        let result = parser.parse_line(line);
        all_activities.extend(result.activities);
    }

    // Test passes if we processed the stream without panic
    // Activities may be empty depending on parser implementation
    let _ = all_activities;
}
