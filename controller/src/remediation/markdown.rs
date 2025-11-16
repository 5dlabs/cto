use crate::remediation::types::CriteriaStatus;
use anyhow::{Context, Result};
use regex::Regex;

/// Markdown parser for extracting structured content from QA feedback comments
pub struct MarkdownParser;

impl MarkdownParser {
    /// Extract acceptance criteria checkboxes from comment body
    pub fn extract_criteria_checkboxes(body: &str) -> Result<Vec<CriteriaStatus>> {
        // Find the "Acceptance Criteria Not Met" section
        let criteria_section = Self::extract_criteria_section(body)?;

        // Parse checkboxes in the section
        Self::parse_checkboxes(&criteria_section)
    }

    /// Extract criteria section content
    fn extract_criteria_section(body: &str) -> Result<String> {
        // Find the start of the section
        let start_marker = "### Acceptance Criteria Not Met";
        let start_pos = body
            .find(start_marker)
            .context("Acceptance Criteria Not Met section not found")?;

        // Start extracting from after the header
        let content_start = start_pos + start_marker.len();
        let remaining_content = &body[content_start..];

        // Find where this section ends (next ### or ** or end of content)
        let end_patterns = ["\n### ", "\n**", "\n***"];
        let mut end_pos = remaining_content.len();

        for pattern in &end_patterns {
            if let Some(pos) = remaining_content.find(pattern) {
                if pos < end_pos {
                    end_pos = pos;
                }
            }
        }

        let section_content = &remaining_content[..end_pos];
        Ok(section_content.trim().to_string())
    }

    /// Parse markdown checkboxes from section content
    fn parse_checkboxes(section: &str) -> Result<Vec<CriteriaStatus>> {
        let mut criteria = Vec::new();
        let mut line_number = 0;

        // Regex for matching markdown checkboxes: - [ ] or - [x] or - [X]
        let checkbox_regex = Regex::new(r"(?m)^\s*-\s*\[([ x])\]\s*(.+)$")
            .context("Failed to compile checkbox regex")?;

        for line in section.lines() {
            line_number += 1;

            if let Some(captures) = checkbox_regex.captures(line) {
                let checkbox_state = captures
                    .get(1)
                    .context("Checkbox state not captured")?
                    .as_str();

                let description = captures
                    .get(2)
                    .context("Checkbox description not captured")?
                    .as_str()
                    .trim()
                    .to_string();

                // Skip empty descriptions
                if description.is_empty() {
                    continue;
                }

                let completed = match checkbox_state {
                    "x" | "X" => true,
                    " " | "" => false,
                    _ => false, // Treat unknown states as uncompleted
                };

                criteria.push(CriteriaStatus {
                    description,
                    completed,
                    line_number: Some(line_number),
                });
            }
        }

        if criteria.is_empty() {
            Err(anyhow::anyhow!("No checkboxes found in criteria section"))
        } else {
            Ok(criteria)
        }
    }

    /// Extract only unmet criteria (uncompleted checkboxes)
    pub fn extract_unmet_criteria(body: &str) -> Result<Vec<String>> {
        let criteria = Self::extract_criteria_checkboxes(body)?;

        let unmet: Vec<String> = criteria
            .into_iter()
            .filter(|c| !c.completed)
            .map(|c| c.description)
            .collect();

        if unmet.is_empty() {
            Err(anyhow::anyhow!("All criteria appear to be met"))
        } else {
            Ok(unmet)
        }
    }

    /// Extract all criteria regardless of completion status
    pub fn extract_all_criteria_descriptions(body: &str) -> Result<Vec<String>> {
        let criteria = Self::extract_criteria_checkboxes(body)?;

        Ok(criteria.into_iter().map(|c| c.description).collect())
    }

    /// Get completion statistics for criteria
    pub fn get_criteria_stats(body: &str) -> Result<(usize, usize, usize)> {
        // (total, completed, uncompleted)
        let criteria = Self::extract_criteria_checkboxes(body)?;

        let total = criteria.len();
        let completed = criteria.iter().filter(|c| c.completed).count();
        let uncompleted = total - completed;

        Ok((total, completed, uncompleted))
    }

    /// Check if all criteria are completed
    #[must_use] pub fn are_all_criteria_met(body: &str) -> bool {
        Self::extract_unmet_criteria(body).is_err() // Returns error if all are met
    }

    /// Extract criteria with their completion status
    pub fn extract_criteria_with_status(body: &str) -> Result<Vec<(String, bool)>> {
        let criteria = Self::extract_criteria_checkboxes(body)?;

        Ok(criteria
            .into_iter()
            .map(|c| (c.description, c.completed))
            .collect())
    }

    /// Handle various markdown checkbox formats
    #[must_use] pub fn normalize_checkbox_syntax(line: &str) -> Option<(bool, String)> {
        let line = line.trim();

        // Handle different checkbox formats
        let patterns = [
            (r"^\s*-\s*\[([ x])\]\s*(.+)$", 1, 2),     // - [ ] description
            (r"^\s*\*\s*\[([ x])\]\s*(.+)$", 1, 2),    // * [ ] description
            (r"^\s*\d+\.\s*\[([ x])\]\s*(.+)$", 1, 2), // 1. [ ] description
        ];

        for (pattern, state_idx, desc_idx) in patterns {
            if let Ok(regex) = Regex::new(pattern) {
                if let Some(captures) = regex.captures(line) {
                    let state = captures.get(state_idx)?.as_str();
                    let description = captures.get(desc_idx)?.as_str().trim().to_string();

                    if !description.is_empty() {
                        let completed = matches!(state, "x" | "X");
                        return Some((completed, description));
                    }
                }
            }
        }

        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const SAMPLE_CRITERIA_SECTION: &str = r"### Acceptance Criteria Not Met
- [ ] User authentication works properly
- [x] Password reset functionality is implemented
- [ ] Error messages are user-friendly
- [ ] Form validation prevents invalid inputs

Some other content here";

    const SAMPLE_COMPLEX_SECTION: &str = r"### Acceptance Criteria Not Met

#### Authentication Features
- [ ] Login with email and password
- [x] Social login integration (Google, GitHub)
- [ ] Two-factor authentication
  - [ ] SMS-based 2FA
  - [ ] Authenticator app support

#### User Management
- [ ] User profile editing
- [ ] Account deletion
- [x] Password change functionality

### Next Section
Some other content";

    #[test]
    fn test_extract_criteria_checkboxes() {
        let comment = format!("Some header\n\n{SAMPLE_CRITERIA_SECTION}");
        let result = MarkdownParser::extract_criteria_checkboxes(&comment);

        assert!(result.is_ok());
        let criteria = result.unwrap();
        assert_eq!(criteria.len(), 4);

        // Check first criterion (uncompleted)
        assert_eq!(
            criteria[0].description,
            "User authentication works properly"
        );
        assert!(!criteria[0].completed);

        // Check second criterion (completed)
        assert_eq!(
            criteria[1].description,
            "Password reset functionality is implemented"
        );
        assert!(criteria[1].completed);

        // Check third criterion (uncompleted)
        assert_eq!(criteria[2].description, "Error messages are user-friendly");
        assert!(!criteria[2].completed);

        // Check fourth criterion (uncompleted)
        assert_eq!(
            criteria[3].description,
            "Form validation prevents invalid inputs"
        );
        assert!(!criteria[3].completed);
    }

    #[test]
    fn test_extract_unmet_criteria() {
        let comment = format!("Some header\n\n{SAMPLE_CRITERIA_SECTION}");
        let result = MarkdownParser::extract_unmet_criteria(&comment);

        assert!(result.is_ok());
        let unmet = result.unwrap();
        assert_eq!(unmet.len(), 3);
        assert!(unmet.contains(&"User authentication works properly".to_string()));
        assert!(unmet.contains(&"Error messages are user-friendly".to_string()));
        assert!(unmet.contains(&"Form validation prevents invalid inputs".to_string()));
        assert!(!unmet.contains(&"Password reset functionality is implemented".to_string()));
    }

    #[test]
    fn test_extract_all_criteria_descriptions() {
        let comment = format!("Some header\n\n{SAMPLE_CRITERIA_SECTION}");
        let result = MarkdownParser::extract_all_criteria_descriptions(&comment);

        assert!(result.is_ok());
        let descriptions = result.unwrap();
        assert_eq!(descriptions.len(), 4);
        assert!(descriptions.contains(&"User authentication works properly".to_string()));
        assert!(descriptions.contains(&"Password reset functionality is implemented".to_string()));
    }

    #[test]
    fn test_get_criteria_stats() {
        let comment = format!("Some header\n\n{SAMPLE_CRITERIA_SECTION}");
        let result = MarkdownParser::get_criteria_stats(&comment);

        assert!(result.is_ok());
        let (total, completed, uncompleted) = result.unwrap();
        assert_eq!(total, 4);
        assert_eq!(completed, 1);
        assert_eq!(uncompleted, 3);
    }

    #[test]
    fn test_are_all_criteria_met() {
        let comment = format!("Some header\n\n{SAMPLE_CRITERIA_SECTION}");
        assert!(!MarkdownParser::are_all_criteria_met(&comment));

        // Create a comment with all criteria met
        let all_met_comment = r"### Acceptance Criteria Not Met
- [x] User authentication works properly
- [x] Password reset functionality is implemented";
        let full_comment = format!("Some header\n\n{all_met_comment}");
        assert!(MarkdownParser::are_all_criteria_met(&full_comment));
    }

    #[test]
    fn test_extract_criteria_with_status() {
        let comment = format!("Some header\n\n{SAMPLE_CRITERIA_SECTION}");
        let result = MarkdownParser::extract_criteria_with_status(&comment);

        assert!(result.is_ok());
        let criteria = result.unwrap();
        assert_eq!(criteria.len(), 4);

        // Check specific criteria
        let auth_criterion = criteria
            .iter()
            .find(|(desc, _)| desc == "User authentication works properly");
        assert!(auth_criterion.is_some());
        assert!(!auth_criterion.unwrap().1); // Should be uncompleted

        let password_criterion = criteria
            .iter()
            .find(|(desc, _)| desc == "Password reset functionality is implemented");
        assert!(password_criterion.is_some());
        assert!(password_criterion.unwrap().1); // Should be completed
    }

    #[test]
    fn test_complex_nested_criteria() {
        let comment = format!("Some header\n\n{SAMPLE_COMPLEX_SECTION}");
        let result = MarkdownParser::extract_criteria_checkboxes(&comment);

        assert!(result.is_ok());
        let criteria = result.unwrap();
        assert_eq!(criteria.len(), 8); // Should find all checkboxes including nested ones

        // Check that nested items are found
        let sms_criterion = criteria.iter().find(|c| c.description == "SMS-based 2FA");
        assert!(sms_criterion.is_some());
        assert!(!sms_criterion.unwrap().completed);
    }

    #[test]
    fn test_normalize_checkbox_syntax() {
        // Test standard markdown checkboxes
        assert_eq!(
            MarkdownParser::normalize_checkbox_syntax("- [ ] Task one"),
            Some((false, "Task one".to_string()))
        );
        assert_eq!(
            MarkdownParser::normalize_checkbox_syntax("- [x] Task two"),
            Some((true, "Task two".to_string()))
        );

        // Test asterisk syntax
        assert_eq!(
            MarkdownParser::normalize_checkbox_syntax("* [ ] Task three"),
            Some((false, "Task three".to_string()))
        );

        // Test numbered list syntax
        assert_eq!(
            MarkdownParser::normalize_checkbox_syntax("1. [x] Task four"),
            Some((true, "Task four".to_string()))
        );

        // Test invalid syntax
        assert_eq!(
            MarkdownParser::normalize_checkbox_syntax("Just some text"),
            None
        );
        assert_eq!(MarkdownParser::normalize_checkbox_syntax("- [ ] "), None); // Empty description
    }

    #[test]
    fn test_missing_criteria_section() {
        let comment = "Some comment without criteria section";
        let result = MarkdownParser::extract_criteria_checkboxes(comment);
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("section not found"));
    }

    #[test]
    fn test_empty_criteria_section() {
        let comment = r"### Acceptance Criteria Not Met

### Next Section";
        let result = MarkdownParser::extract_criteria_checkboxes(comment);
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("No checkboxes found"));
    }
}
