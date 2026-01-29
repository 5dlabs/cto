//! Shared log-based initialization parsing.
//!
//! Parses MCP tool initialization from agent log files,
//! extracting tool counts and prefixes from `mcp-check.sh.hbs` output.
//!
//! # Log Format
//!
//! The MCP check script outputs lines like:
//! ```text
//! 🔧 MCP Tools Diagnostics:
//! ───────────────────────────────────────────────────────────────
//!
//! 📦 MCP Client Binary:
//!   ✅ tools binary found: /usr/local/bin/tools
//!   → Version: tools 0.2.33
//!
//! 📋 MCP Configuration:
//!   ✅ Config found: /mcp-config/mcp.json
//!   ✅ Valid JSON
//!   → Transport: http
//!   → URL: http://cto-tools.cto.svc.cluster.local:3000/mcp
//!   → Available tools: 15
//!   → Tool prefixes:
//!      • filesystem
//!      • github
//!      • linear
//! ```

use regex::Regex;
use std::sync::LazyLock;

use super::InitInfo;

/// Regex for "Available tools: N" line
static TOOLS_COUNT_RE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"Available tools:\s*(\d+)").expect("Invalid tools count regex"));

/// Regex for tool prefix lines "• prefix" or "· prefix"
static TOOL_PREFIX_RE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"[•·]\s*(\S+)").expect("Invalid tool prefix regex"));

/// Regex for model name in various formats
static MODEL_RE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r#"(?i)(?:model|using|with)\s*[:=]?\s*["\']?([a-z0-9._-]+(?:-[a-z0-9._-]+)*)["\']?"#)
        .expect("Invalid model regex")
});

/// Regex for "Starting with model" or similar
static STARTING_MODEL_RE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"(?i)Starting\s+with\s+([a-z0-9._-]+(?:-[a-z0-9._-]+)*)")
        .expect("Invalid starting model regex")
});

/// Parse MCP initialization info from log content.
///
/// Extracts:
/// - Tool count from "Available tools: N" line
/// - Tool prefixes from "• prefix" lines
/// - Model name if found in log
///
/// # Arguments
/// * `log` - Full log file content
///
/// # Returns
/// * `Some(InitInfo)` if any initialization info was found
/// * `None` if no relevant info could be extracted
#[must_use]
pub fn parse_mcp_init_from_log(log: &str) -> Option<InitInfo> {
    let mut info = InitInfo::default();
    let mut found_anything = false;

    // Look for "Available tools: N" line
    if let Some(cap) = TOOLS_COUNT_RE.captures(log) {
        if let Ok(count) = cap[1].parse::<usize>() {
            info.tool_count = count;
            found_anything = true;
        }
    }

    // Look for tool prefix lines
    // First, find the "Tool prefixes:" section if it exists
    let prefix_section = if let Some(idx) = log.find("Tool prefixes:") {
        // Get content after "Tool prefixes:" up to the next major section
        let remainder = &log[idx..];
        remainder
            .lines()
            .skip(1) // Skip the "Tool prefixes:" line itself
            .take_while(|line| line.trim().starts_with('•') || line.trim().starts_with('·'))
            .collect::<Vec<_>>()
            .join("\n")
    } else {
        String::new()
    };

    // Parse tool prefixes from the section
    for cap in TOOL_PREFIX_RE.captures_iter(&prefix_section) {
        let prefix = cap[1].to_string();
        if !info.tool_names.contains(&prefix) {
            info.tool_names.push(prefix);
            found_anything = true;
        }
    }

    // Look for model name
    if let Some(cap) = STARTING_MODEL_RE.captures(log) {
        info.model = Some(cap[1].to_string());
        found_anything = true;
    } else if let Some(cap) = MODEL_RE.captures(log) {
        info.model = Some(cap[1].to_string());
        found_anything = true;
    }

    // If tool_count is 0 but we have tool names, update count
    if info.tool_count == 0 && !info.tool_names.is_empty() {
        info.tool_count = info.tool_names.len();
    }

    if found_anything {
        Some(info)
    } else {
        None
    }
}

/// Parse model name from a log line.
///
/// Handles formats like:
/// - "Starting with claude-opus-4"
/// - "model: gemini-2.0-flash"
/// - "Using gpt-4o"
#[must_use]
pub fn extract_model_from_line(line: &str) -> Option<String> {
    if let Some(cap) = STARTING_MODEL_RE.captures(line) {
        return Some(cap[1].to_string());
    }
    if let Some(cap) = MODEL_RE.captures(line) {
        return Some(cap[1].to_string());
    }
    None
}

/// Extract tool count from a single line.
#[must_use]
pub fn extract_tools_count_from_line(line: &str) -> Option<usize> {
    TOOLS_COUNT_RE
        .captures(line)
        .and_then(|cap| cap[1].parse().ok())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_mcp_init_basic() {
        let log = r"
🔧 MCP Tools Diagnostics:
───────────────────────────────────────────────────────────────

📦 MCP Client Binary:
  ✅ tools binary found: /usr/local/bin/tools

📋 MCP Configuration:
  ✅ Config found: /mcp-config/mcp.json
  → Available tools: 15
  → Tool prefixes:
     • filesystem
     • github
     • linear
";
        let info = parse_mcp_init_from_log(log).unwrap();
        assert_eq!(info.tool_count, 15);
        assert_eq!(info.tool_names.len(), 3);
        assert!(info.tool_names.contains(&"filesystem".to_string()));
        assert!(info.tool_names.contains(&"github".to_string()));
        assert!(info.tool_names.contains(&"linear".to_string()));
    }

    #[test]
    fn test_parse_mcp_init_with_model() {
        let log = "Starting with claude-opus-4\n→ Available tools: 10";
        let info = parse_mcp_init_from_log(log).unwrap();
        assert_eq!(info.model, Some("claude-opus-4".to_string()));
        assert_eq!(info.tool_count, 10);
    }

    #[test]
    fn test_extract_model() {
        assert_eq!(
            extract_model_from_line("Starting with gemini-2.0-flash"),
            Some("gemini-2.0-flash".to_string())
        );
        assert_eq!(
            extract_model_from_line("model: gpt-4o"),
            Some("gpt-4o".to_string())
        );
        assert_eq!(
            extract_model_from_line("Using codex-3.5"),
            Some("codex-3.5".to_string())
        );
        assert_eq!(extract_model_from_line("This is just random text"), None);
    }

    #[test]
    fn test_extract_tools_count() {
        assert_eq!(
            extract_tools_count_from_line("→ Available tools: 25"),
            Some(25)
        );
        assert_eq!(extract_tools_count_from_line("Available tools: 0"), Some(0));
        assert_eq!(extract_tools_count_from_line("No tools here"), None);
    }

    #[test]
    fn test_parse_empty_log() {
        assert!(parse_mcp_init_from_log("").is_none());
        assert!(parse_mcp_init_from_log("Some random log content").is_none());
    }

    #[test]
    fn test_tool_prefixes_with_different_bullets() {
        let log = r"
Tool prefixes:
   • prefix1
   · prefix2
     • prefix3
";
        let info = parse_mcp_init_from_log(log).unwrap();
        assert_eq!(info.tool_names.len(), 3);
    }
}
