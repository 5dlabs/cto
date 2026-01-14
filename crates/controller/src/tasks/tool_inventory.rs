//! Tool Inventory Diff for Healer monitoring.
//!
//! Compares declared tools in CTO config vs tools actually available at runtime.
//! This is used by Healer to detect config-to-CLI tool mismatches.

use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use tracing::{info, warn};

use super::tool_catalog::resolve_tool_name;

/// Result of comparing declared vs available tools.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolInventoryDiff {
    /// Tools declared in CTO config that resolved successfully
    pub resolved_tools: Vec<String>,
    /// Tools declared but not found in catalog (potential issues)
    pub unresolved_tools: Vec<String>,
    /// Total tools declared
    pub declared_count: usize,
    /// Whether all tools were successfully resolved
    pub all_resolved: bool,
    /// Agent name this diff is for
    pub agent: String,
}

/// Log the tool inventory status for an agent.
///
/// Returns a `ToolInventoryDiff` that summarizes the comparison.
pub fn log_tool_inventory(agent_name: &str, declared_tools: &[String]) -> ToolInventoryDiff {
    let mut resolved_tools = Vec::new();
    let mut unresolved_tools = Vec::new();

    for tool in declared_tools {
        match resolve_tool_name(tool) {
            Some(resolved) => {
                resolved_tools.push(resolved);
            }
            None => {
                unresolved_tools.push(tool.clone());
            }
        }
    }

    let all_resolved = unresolved_tools.is_empty();
    let declared_count = declared_tools.len();

    // Log the inventory status
    if all_resolved {
        info!(
            agent = %agent_name,
            declared = declared_count,
            resolved = resolved_tools.len(),
            "✅ All declared tools resolved successfully"
        );
    } else {
        warn!(
            agent = %agent_name,
            declared = declared_count,
            resolved = resolved_tools.len(),
            unresolved = unresolved_tools.len(),
            unresolved_tools = ?unresolved_tools,
            "⚠️ Some declared tools could not be resolved"
        );
    }

    // Log individual tools for debugging
    for (i, tool) in resolved_tools.iter().enumerate() {
        info!(
            agent = %agent_name,
            tool_index = i + 1,
            tool_name = %tool,
            "Tool resolved"
        );
    }

    ToolInventoryDiff {
        resolved_tools,
        unresolved_tools,
        declared_count,
        all_resolved,
        agent: agent_name.to_string(),
    }
}

/// Validate that expected tools are present in the actual tools list.
///
/// Returns a list of missing tools.
pub fn validate_expected_tools(expected: &[String], actual: &[String]) -> Vec<String> {
    let actual_set: HashSet<&String> = actual.iter().collect();
    let mut missing = Vec::new();

    for tool in expected {
        // Try to resolve the expected tool name first
        let resolved = resolve_tool_name(tool).unwrap_or_else(|| tool.clone());
        if !actual_set.contains(&resolved) && !actual_set.contains(tool) {
            missing.push(tool.clone());
        }
    }

    if !missing.is_empty() {
        warn!(
            missing_count = missing.len(),
            missing_tools = ?missing,
            "Expected tools not found in actual tools list"
        );
    }

    missing
}

/// Format a tool inventory diff as a human-readable string.
#[must_use]
pub fn format_inventory_diff(diff: &ToolInventoryDiff) -> String {
    use std::fmt::Write;

    let mut output = String::new();

    let _ = writeln!(output, "Tool Inventory for {}", diff.agent);
    let _ = writeln!(output, "  Declared: {} tools", diff.declared_count);
    let _ = writeln!(output, "  Resolved: {} tools", diff.resolved_tools.len());
    let _ = writeln!(
        output,
        "  Unresolved: {} tools",
        diff.unresolved_tools.len()
    );
    let _ = writeln!(
        output,
        "  Status: {}",
        if diff.all_resolved {
            "✅ OK"
        } else {
            "⚠️ Issues"
        }
    );

    if !diff.unresolved_tools.is_empty() {
        output.push_str("\nUnresolved tools:\n");
        for tool in &diff.unresolved_tools {
            let _ = writeln!(output, "  - {tool}");
        }
    }

    output
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_log_tool_inventory_all_resolved() {
        // Note: In tests, the tool catalog won't be loaded from the real path
        // so resolve_tool_name will return the tool unchanged
        let tools = vec![
            "memory_create_entities".to_string(),
            "brave_search_brave_web_search".to_string(),
        ];

        let diff = log_tool_inventory("rex", &tools);

        // Since catalog won't be loaded in tests, all tools should "resolve"
        // (they're returned unchanged)
        assert_eq!(diff.agent, "rex");
        assert_eq!(diff.declared_count, 2);
        assert!(diff.all_resolved);
    }

    #[test]
    fn test_validate_expected_tools_finds_missing() {
        let expected = vec!["tool_a".to_string(), "tool_b".to_string()];
        let actual = vec!["tool_a".to_string()];

        let missing = validate_expected_tools(&expected, &actual);

        assert_eq!(missing.len(), 1);
        assert_eq!(missing[0], "tool_b");
    }

    #[test]
    fn test_validate_expected_tools_all_present() {
        let expected = vec!["tool_a".to_string(), "tool_b".to_string()];
        let actual = vec![
            "tool_a".to_string(),
            "tool_b".to_string(),
            "tool_c".to_string(),
        ];

        let missing = validate_expected_tools(&expected, &actual);

        assert!(missing.is_empty());
    }

    #[test]
    fn test_format_inventory_diff() {
        let diff = ToolInventoryDiff {
            resolved_tools: vec!["tool_a".to_string()],
            unresolved_tools: vec!["missing_tool".to_string()],
            declared_count: 2,
            all_resolved: false,
            agent: "blaze".to_string(),
        };

        let formatted = format_inventory_diff(&diff);

        assert!(formatted.contains("blaze"));
        assert!(formatted.contains("Declared: 2"));
        assert!(formatted.contains("Resolved: 1"));
        assert!(formatted.contains("Unresolved: 1"));
        assert!(formatted.contains("missing_tool"));
    }
}
