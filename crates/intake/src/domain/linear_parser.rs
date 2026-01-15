//! Linear issue parser for extracting task content.
//!
//! This module provides functionality to parse Linear issues and extract
//! structured task information including title, description, and acceptance criteria.

use serde::{Deserialize, Serialize};

/// Parsed task content from a Linear issue.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParsedLinearTask {
    /// Task title (from Linear issue title).
    pub title: String,

    /// Task description (parsed from Linear issue description).
    pub description: String,

    /// Acceptance criteria (extracted from Linear issue description).
    pub acceptance_criteria: Vec<String>,

    /// Test strategy hints (extracted from Linear issue description).
    #[serde(default)]
    pub test_strategy: Option<String>,

    /// Priority (from Linear issue priority: 1=urgent, 2=high, 3=normal, 4=low).
    #[serde(default)]
    pub priority: Option<i32>,

    /// Agent hint (extracted from Linear labels like `claude:opus` or description).
    #[serde(default)]
    pub agent_hint: Option<String>,
}

/// Parse a Linear issue into structured task content.
///
/// This function extracts task information from a Linear issue:
/// - Title: Direct from issue title
/// - Description: Main body before any structured sections
/// - Acceptance Criteria: Items under "## Acceptance Criteria" or "### AC" headings
/// - Test Strategy: Content under "## Test Strategy" or "### Testing" headings
/// - Agent Hint: From labels matching pattern `cli:model` (e.g., `claude:opus`)
///
/// # Arguments
/// * `title` - The Linear issue title
/// * `description` - The Linear issue description (markdown)
/// * `labels` - Linear labels on the issue
/// * `priority` - Linear priority (1-4, where 1 is urgent)
///
/// # Returns
/// A `ParsedLinearTask` with extracted content.
#[must_use]
pub fn parse_linear_issue(
    title: &str,
    description: Option<&str>,
    labels: &[String],
    priority: i32,
) -> ParsedLinearTask {
    let description_text = description.unwrap_or("");

    // Parse description sections
    let (main_description, acceptance_criteria, test_strategy) =
        parse_description_sections(description_text);

    // Extract agent hint from labels (format: cli:model, e.g., claude:opus)
    let agent_hint = extract_agent_hint_from_labels(labels);

    ParsedLinearTask {
        title: title.to_string(),
        description: main_description,
        acceptance_criteria,
        test_strategy,
        priority: if priority > 0 { Some(priority) } else { None },
        agent_hint,
    }
}

/// Parse description into sections.
///
/// Extracts:
/// - Main description (everything before first recognized heading)
/// - Acceptance criteria (bullet points under AC headings)
/// - Test strategy (content under testing headings)
#[derive(PartialEq, Eq)]
enum Section {
    Main,
    AcceptanceCriteria,
    TestStrategy,
    Other,
}

fn parse_description_sections(description: &str) -> (String, Vec<String>, Option<String>) {
    let mut main_desc = String::new();
    let mut acceptance_criteria = Vec::new();
    let mut test_strategy = None;
    let mut current_section = Section::Main;
    let mut test_strategy_lines = Vec::new();

    for line in description.lines() {
        let trimmed = line.trim();
        let lower = trimmed.to_lowercase();

        // Detect section headers
        if trimmed.starts_with('#') {
            if lower.contains("acceptance criteria")
                || lower.contains("acceptance")
                || lower.contains("## ac")
                || lower.contains("### ac")
            {
                current_section = Section::AcceptanceCriteria;
                continue;
            } else if lower.contains("test strategy")
                || lower.contains("testing")
                || lower.contains("test plan")
            {
                current_section = Section::TestStrategy;
                continue;
            } else if lower.contains("description") {
                current_section = Section::Main;
                continue;
            }
            // Other heading - skip section
            current_section = Section::Other;
            continue;
        }

        match current_section {
            Section::Main => {
                if !main_desc.is_empty() || !trimmed.is_empty() {
                    if !main_desc.is_empty() {
                        main_desc.push('\n');
                    }
                    main_desc.push_str(line);
                }
            }
            Section::AcceptanceCriteria => {
                // Extract bullet points
                if let Some(item) = extract_bullet_item(trimmed) {
                    if !item.is_empty() {
                        acceptance_criteria.push(item);
                    }
                }
            }
            Section::TestStrategy => {
                test_strategy_lines.push(line.to_string());
            }
            Section::Other => {
                // Skip content in unrecognized sections
            }
        }
    }

    // Combine test strategy lines
    if !test_strategy_lines.is_empty() {
        let strategy = test_strategy_lines.join("\n").trim().to_string();
        if !strategy.is_empty() {
            test_strategy = Some(strategy);
        }
    }

    // Clean up main description
    let main_desc = main_desc.trim().to_string();

    (main_desc, acceptance_criteria, test_strategy)
}

/// Extract bullet item content from a line.
///
/// Handles various bullet formats:
/// - `- item`
/// - `* item`
/// - `• item`
/// - `- [ ] item` (checkbox)
/// - `- [x] item` (checked checkbox)
/// - `1. item` (numbered)
fn extract_bullet_item(line: &str) -> Option<String> {
    let trimmed = line.trim();

    // Check for bullet markers
    let content = if trimmed.starts_with("- [ ] ") {
        Some(trimmed.strip_prefix("- [ ] ")?.to_string())
    } else if trimmed.starts_with("- [x] ") || trimmed.starts_with("- [X] ") {
        Some(trimmed[6..].to_string())
    } else if trimmed.starts_with("- ") {
        Some(trimmed.strip_prefix("- ")?.to_string())
    } else if trimmed.starts_with("* ") {
        Some(trimmed.strip_prefix("* ")?.to_string())
    } else if trimmed.starts_with("• ") {
        Some(trimmed.strip_prefix("• ")?.to_string())
    } else if let Some(rest) = trimmed.strip_prefix(|c: char| c.is_ascii_digit()) {
        // Handle numbered lists like "1. item"
        rest.strip_prefix(". ").map(ToString::to_string)
    } else {
        None
    };

    content.map(|s| s.trim().to_string())
}

/// Extract agent hint from Linear labels.
///
/// Looks for labels in the format `cli:model` (e.g., `claude:opus`, `cursor:sonnet`).
/// Returns the model part if found (e.g., `opus`, `sonnet`).
fn extract_agent_hint_from_labels(labels: &[String]) -> Option<String> {
    // Known CLI prefixes
    const CLI_PREFIXES: &[&str] = &["claude:", "cursor:", "codex:", "opencode:", "gemini:"];

    for label in labels {
        let lower = label.to_lowercase();
        for prefix in CLI_PREFIXES {
            if lower.starts_with(prefix) {
                // Extract the model part after the colon
                // But for agent routing, we want the agent name, not the model
                // So we return the full label for now
                return Some(label.clone());
            }
        }

        // Also check for explicit agent labels like `agent:bolt`, `agent:rex`
        if let Some(agent) = lower.strip_prefix("agent:") {
            return Some(agent.to_string());
        }
    }

    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_simple_issue() {
        let result = parse_linear_issue(
            "Implement user authentication",
            Some("Add login and logout functionality to the application."),
            &[],
            2,
        );

        assert_eq!(result.title, "Implement user authentication");
        assert_eq!(
            result.description,
            "Add login and logout functionality to the application."
        );
        assert!(result.acceptance_criteria.is_empty());
        assert_eq!(result.priority, Some(2));
    }

    #[test]
    fn test_parse_issue_with_acceptance_criteria() {
        let description = r"
Implement the login form component.

## Acceptance Criteria
- User can enter email and password
- Form validates input before submission
- Error messages are displayed for invalid input
- Successful login redirects to dashboard
";

        let result = parse_linear_issue("Login form", Some(description), &[], 3);

        assert_eq!(result.title, "Login form");
        assert!(result.description.contains("login form component"));
        assert_eq!(result.acceptance_criteria.len(), 4);
        assert!(result.acceptance_criteria[0].contains("email and password"));
    }

    #[test]
    fn test_parse_issue_with_test_strategy() {
        let description = r"
Add unit tests for the auth module.

## Test Strategy
- Unit tests for login function
- Integration tests for auth flow
- E2E tests for login page
";

        let result = parse_linear_issue("Auth tests", Some(description), &[], 3);

        assert!(result.test_strategy.is_some());
        let strategy = result.test_strategy.unwrap();
        assert!(strategy.contains("Unit tests"));
    }

    #[test]
    fn test_extract_agent_hint_from_labels() {
        let labels = vec![
            "bug".to_string(),
            "claude:opus".to_string(),
            "priority".to_string(),
        ];

        let hint = extract_agent_hint_from_labels(&labels);
        assert_eq!(hint, Some("claude:opus".to_string()));
    }

    #[test]
    fn test_extract_agent_hint_explicit() {
        let labels = vec!["agent:bolt".to_string(), "feature".to_string()];

        let hint = extract_agent_hint_from_labels(&labels);
        assert_eq!(hint, Some("bolt".to_string()));
    }

    #[test]
    fn test_extract_bullet_items() {
        assert_eq!(
            extract_bullet_item("- simple item"),
            Some("simple item".to_string())
        );
        assert_eq!(
            extract_bullet_item("* asterisk item"),
            Some("asterisk item".to_string())
        );
        assert_eq!(
            extract_bullet_item("- [ ] checkbox"),
            Some("checkbox".to_string())
        );
        assert_eq!(
            extract_bullet_item("- [x] checked"),
            Some("checked".to_string())
        );
        assert_eq!(
            extract_bullet_item("1. numbered"),
            Some("numbered".to_string())
        );
        assert_eq!(extract_bullet_item("not a bullet"), None);
    }

    #[test]
    fn test_parse_issue_no_description() {
        let result = parse_linear_issue("Task title", None, &[], 0);

        assert_eq!(result.title, "Task title");
        assert!(result.description.is_empty());
        assert!(result.acceptance_criteria.is_empty());
        assert!(result.priority.is_none());
    }
}
