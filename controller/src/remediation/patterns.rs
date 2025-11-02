use crate::remediation::types::{IssueType, Severity};
use anyhow::{Context, Result};
use lazy_static::lazy_static;
use regex::Regex;

/// Pattern extractor for parsing structured feedback from QA comments
pub struct PatternExtractor;

lazy_static! {
    /// Pattern for extracting Issue Type: **Issue Type**: [value]
    static ref ISSUE_TYPE_PATTERN: Regex =
        Regex::new(r"(?m)^\s*\*\*Issue Type\*\*:\s*\[(.*?)\]").unwrap();

    /// Pattern for extracting Severity: **Severity**: [value]
    static ref SEVERITY_PATTERN: Regex =
        Regex::new(r"(?m)^\s*\*\*Severity\*\*:\s*\[(.*?)\]").unwrap();

    /// Pattern for extracting Description section
    static ref DESCRIPTION_PATTERN: Regex =
        Regex::new(r"(?ms)### Description\s*\n(.*?)(?:\n### |\n\*\*|$)")
            .context("Failed to compile description pattern")
            .unwrap();

    /// Pattern for extracting Steps to Reproduce section
    static ref STEPS_PATTERN: Regex =
        Regex::new(r"(?ms)### Steps to Reproduce.*?\n(.*(?:\n###|\n\*\*|$))")
            .context("Failed to compile steps pattern")
            .unwrap();

    /// Pattern for extracting Expected behavior: **Expected**: value
    static ref EXPECTED_PATTERN: Regex =
        Regex::new(r"(?m)^\s*-?\s*\*\*Expected\*\*:\s*(.+)$")
            .context("Failed to compile expected pattern")
            .unwrap();

    /// Pattern for extracting Actual behavior: **Actual**: value
    static ref ACTUAL_PATTERN: Regex =
        Regex::new(r"(?m)^\s*-?\s*\*\*Actual\*\*:\s*(.+)$")
            .context("Failed to compile actual pattern")
            .unwrap();

    /// Pattern for extracting Expected vs Actual section
    static ref EXPECTED_ACTUAL_SECTION_PATTERN: Regex =
        Regex::new(r"(?ms)### Expected vs Actual.*?\n(.*?)(?:\n### |\n\*\*|$)")
            .context("Failed to compile expected actual section pattern")
            .unwrap();
}

impl PatternExtractor {
    /// Extract Issue Type from comment body
    pub fn extract_issue_type(body: &str) -> Result<IssueType> {
        let captures = ISSUE_TYPE_PATTERN
            .captures(body)
            .context("Issue Type not found in comment")?;

        let issue_type_str = captures
            .get(1)
            .context("Issue Type value not captured")?
            .as_str()
            .trim();

        Self::parse_issue_type(issue_type_str)
    }

    /// Extract Severity from comment body
    pub fn extract_severity(body: &str) -> Result<Severity> {
        let captures = SEVERITY_PATTERN
            .captures(body)
            .context("Severity not found in comment")?;

        let severity_str = captures
            .get(1)
            .context("Severity value not captured")?
            .as_str()
            .trim();

        Self::parse_severity(severity_str)
    }

    /// Extract Description section from comment body
    pub fn extract_description(body: &str) -> Result<String> {
        let captures = DESCRIPTION_PATTERN
            .captures(body)
            .context("Description section not found")?;

        let description = captures
            .get(1)
            .context("Description content not captured")?
            .as_str()
            .trim()
            .to_string();

        if description.is_empty() {
            Err(anyhow::anyhow!("Description section is empty"))
        } else {
            Ok(description)
        }
    }

    /// Extract reproduction steps from comment body
    pub fn extract_reproduction_steps(body: &str) -> Result<Vec<String>> {
        let captures = STEPS_PATTERN
            .captures(body)
            .context("Steps to Reproduce section not found")?;

        let steps_text = captures
            .get(1)
            .context("Steps content not captured")?
            .as_str();

        Self::parse_steps(steps_text)
    }

    /// Extract expected and actual behavior from comment body
    pub fn extract_expected_actual(body: &str) -> (Option<String>, Option<String>) {
        let expected = Self::extract_expected_behavior(body);
        let actual = Self::extract_actual_behavior(body);

        (expected, actual)
    }

    /// Extract expected behavior using multiple strategies
    fn extract_expected_behavior(body: &str) -> Option<String> {
        // Try the main expected pattern first
        if let Some(expected) = EXPECTED_PATTERN
            .captures(body)
            .and_then(|c| c.get(1))
            .map(|m| m.as_str().trim().to_string())
            .filter(|s| !s.is_empty())
        {
            return Some(expected);
        }

        // Try extracting from Expected vs Actual section
        if let Some(section) = EXPECTED_ACTUAL_SECTION_PATTERN
            .captures(body)
            .and_then(|c| c.get(1))
            .map(|m| m.as_str())
        {
            // Look for Expected within the section
            for line in section.lines() {
                let trimmed = line.trim();
                if trimmed.starts_with("- **Expected**:") || trimmed.starts_with("**Expected**:") {
                    let value = trimmed
                        .split_once(':')
                        .map(|x| x.1)
                        .map(|s| s.trim().to_string())
                        .filter(|s| !s.is_empty());
                    if value.is_some() {
                        return value;
                    }
                }
            }
        }

        None
    }

    /// Extract actual behavior using multiple strategies
    fn extract_actual_behavior(body: &str) -> Option<String> {
        // Try the main actual pattern first
        if let Some(actual) = ACTUAL_PATTERN
            .captures(body)
            .and_then(|c| c.get(1))
            .map(|m| m.as_str().trim().to_string())
            .filter(|s| !s.is_empty())
        {
            return Some(actual);
        }

        // Try extracting from Expected vs Actual section
        if let Some(section) = EXPECTED_ACTUAL_SECTION_PATTERN
            .captures(body)
            .and_then(|c| c.get(1))
            .map(|m| m.as_str())
        {
            // Look for Actual within the section
            for line in section.lines() {
                let trimmed = line.trim();
                if trimmed.starts_with("- **Actual**:") || trimmed.starts_with("**Actual**:") {
                    let value = trimmed
                        .split_once(':')
                        .map(|x| x.1)
                        .map(|s| s.trim().to_string())
                        .filter(|s| !s.is_empty());
                    if value.is_some() {
                        return value;
                    }
                }
            }
        }

        None
    }

    /// Parse issue type string into enum
    fn parse_issue_type(s: &str) -> Result<IssueType> {
        match s {
            "Bug" => Ok(IssueType::Bug),
            "Missing Feature" => Ok(IssueType::MissingFeature),
            "Regression" => Ok(IssueType::Regression),
            "Performance" => Ok(IssueType::Performance),
            _ => Err(anyhow::anyhow!("Unknown issue type: {s}")),
        }
    }

    /// Parse severity string into enum
    fn parse_severity(s: &str) -> Result<Severity> {
        match s {
            "Critical" => Ok(Severity::Critical),
            "High" => Ok(Severity::High),
            "Medium" => Ok(Severity::Medium),
            "Low" => Ok(Severity::Low),
            _ => Err(anyhow::anyhow!("Unknown severity: {s}")),
        }
    }

    /// Parse steps text into vector of step strings
    fn parse_steps(steps_text: &str) -> Result<Vec<String>> {
        let steps: Vec<String> = steps_text
            .lines()
            .filter_map(|line| {
                let trimmed = line.trim();

                // Skip empty lines
                if trimmed.is_empty() {
                    return None;
                }

                // Skip lines that start with ### (section headers)
                if trimmed.starts_with("###") {
                    return None;
                }

                // Skip lines that start with - ** (expected/actual format)
                if trimmed.starts_with("- **") {
                    return None;
                }

                // Only process lines that start with a number followed by a dot
                if !trimmed.chars().next().is_some_and(|c| c.is_ascii_digit()) {
                    return None;
                }

                if !trimmed.contains('.') {
                    return None;
                }

                // Remove leading number and dot/space
                let step = trimmed
                    .split_once('.')
                    .map(|(_, rest)| rest.trim())
                    .unwrap_or(trimmed);

                Some(step.to_string())
            })
            .collect();

        if steps.is_empty() {
            Err(anyhow::anyhow!("No reproduction steps found"))
        } else {
            Ok(steps)
        }
    }

    /// Check if comment contains actionable feedback marker
    pub fn is_actionable_feedback(body: &str) -> bool {
        body.contains("🔴 Required Changes")
    }

    /// Extract optional description (returns None if not found)
    pub fn extract_description_optional(body: &str) -> Option<String> {
        Self::extract_description(body).ok()
    }

    /// Extract optional reproduction steps (returns None if not found)
    pub fn extract_reproduction_steps_optional(body: &str) -> Option<Vec<String>> {
        Self::extract_reproduction_steps(body).ok()
    }

    /// Extract optional issue type (returns None if not found)
    pub fn extract_issue_type_optional(body: &str) -> Option<IssueType> {
        Self::extract_issue_type(body).ok()
    }

    /// Extract optional severity (returns None if not found)
    pub fn extract_severity_optional(body: &str) -> Option<Severity> {
        Self::extract_severity(body).ok()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const SAMPLE_COMMENT: &str = r"🔴 Required Changes
**Issue Type**: [Bug]
**Severity**: [High]

### Description
The login button is not working properly when users click it.

### Acceptance Criteria Not Met
- [ ] User authentication works
- [x] Password reset functions

### Steps to Reproduce
1. Navigate to login page
2. Enter valid credentials
3. Click login button

### Expected vs Actual
- **Expected**: User should be logged in and redirected to dashboard
- **Actual**: Page refreshes without login attempt";

    #[test]
    fn test_extract_issue_type() {
        let result = PatternExtractor::extract_issue_type(SAMPLE_COMMENT);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), IssueType::Bug);
    }

    #[test]
    fn test_extract_severity() {
        let result = PatternExtractor::extract_severity(SAMPLE_COMMENT);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), Severity::High);
    }

    #[test]
    fn test_extract_description() {
        let result = PatternExtractor::extract_description(SAMPLE_COMMENT);
        assert!(result.is_ok());
        let desc = result.unwrap();
        assert!(desc.contains("login button"));
        assert!(desc.contains("working properly"));
    }

    #[test]
    fn test_extract_reproduction_steps() {
        let result = PatternExtractor::extract_reproduction_steps(SAMPLE_COMMENT);
        assert!(result.is_ok());
        let steps = result.unwrap();
        assert_eq!(steps.len(), 3);
        assert!(steps[0].contains("Navigate to login page"));
        assert!(steps[1].contains("Enter valid credentials"));
        assert!(steps[2].contains("Click login button"));
    }

    #[test]
    fn test_extract_expected_actual() {
        let (expected, actual) = PatternExtractor::extract_expected_actual(SAMPLE_COMMENT);
        assert!(expected.is_some());
        assert!(actual.is_some());
        assert!(expected.unwrap().contains("logged in"));
        assert!(actual.unwrap().contains("refreshes"));
    }

    #[test]
    fn test_is_actionable_feedback() {
        assert!(PatternExtractor::is_actionable_feedback(SAMPLE_COMMENT));
        assert!(!PatternExtractor::is_actionable_feedback(
            "Just a regular comment"
        ));
    }

    #[test]
    fn test_parse_issue_type_invalid() {
        let result = PatternExtractor::parse_issue_type("InvalidType");
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Unknown issue type"));
    }

    #[test]
    fn test_parse_severity_invalid() {
        let result = PatternExtractor::parse_severity("InvalidSeverity");
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Unknown severity"));
    }

    #[test]
    fn test_parse_steps_empty() {
        let result = PatternExtractor::parse_steps("");
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("No reproduction steps"));
    }

    #[test]
    fn test_parse_steps_with_numbers() {
        let steps_text = "1. First step\n2. Second step\n3. Third step";
        let result = PatternExtractor::parse_steps(steps_text);
        assert!(result.is_ok());
        let steps = result.unwrap();
        assert_eq!(steps.len(), 3);
        assert_eq!(steps[0], "First step");
        assert_eq!(steps[1], "Second step");
        assert_eq!(steps[2], "Third step");
    }

    #[test]
    fn test_missing_issue_type() {
        let comment_without_type = "Some comment without issue type";
        let result = PatternExtractor::extract_issue_type(comment_without_type);
        assert!(result.is_err());
    }

    #[test]
    fn test_missing_severity() {
        let comment_without_severity = "Some comment without severity";
        let result = PatternExtractor::extract_severity(comment_without_severity);
        assert!(result.is_err());
    }
}
