use anyhow::{Context, Result};
use chrono::Utc;
use tracing::info;

use crate::remediation::{
    auth::AuthorValidator,
    error::{ErrorContext, ParseError, ParseResult},
    markdown::MarkdownParser,
    patterns::PatternExtractor,
    types::{CriteriaStatus, FeedbackMetadata, IssueType, Severity, StructuredFeedback},
};

/// Main feedback parser that orchestrates all parsing components
#[derive(Debug)]
pub struct FeedbackParser {
    /// Author validation component
    author_validator: AuthorValidator,
    /// Whether to enable detailed logging
    detailed_logging: bool,
    /// Maximum comment size to process (in bytes)
    max_comment_size: usize,
}

impl FeedbackParser {
    /// Create a new feedback parser with default settings
    #[must_use]
    pub fn new() -> Self {
        Self {
            author_validator: AuthorValidator::new(),
            detailed_logging: true,
            max_comment_size: 10 * 1024 * 1024, // 10MB default
        }
    }

    /// Create parser with custom author validator
    #[must_use]
    pub fn with_validator(validator: AuthorValidator) -> Self {
        Self {
            author_validator: validator,
            detailed_logging: true,
            max_comment_size: 10 * 1024 * 1024,
        }
    }

    /// Create parser with custom settings
    #[must_use]
    pub fn with_config(
        validator: AuthorValidator,
        detailed_logging: bool,
        max_comment_size: usize,
    ) -> Self {
        Self {
            author_validator: validator,
            detailed_logging,
            max_comment_size,
        }
    }

    /// Parse a complete feedback comment into structured data
    pub fn parse_comment(
        &self,
        comment_body: &str,
        author: &str,
        comment_id: u64,
        pr_number: u32,
        task_id: &str,
    ) -> ParseResult<StructuredFeedback> {
        let start_time = std::time::Instant::now();

        if self.detailed_logging {
            info!(
                "Starting feedback parsing for comment {} from author '{}'",
                comment_id, author
            );
        }

        // Validate input size
        self.validate_comment_size(comment_body)?;

        // Step 1: Check if this is actionable feedback
        if !PatternExtractor::is_actionable_feedback(comment_body) {
            let error = ParseError::NotActionableFeedback;
            self.log_error(&ErrorContext::new(error.clone()).with_comment(
                comment_id,
                pr_number,
                author.to_string(),
            ));
            return Err(error);
        }

        // Step 2: Validate author
        if let Err(auth_error) = self.author_validator.validate_author(author) {
            let error = ParseError::UnauthorizedAuthor {
                author: author.to_string(),
            };
            self.log_error(
                &ErrorContext::new(error.clone())
                    .with_comment(comment_id, pr_number, author.to_string())
                    .with_info("auth_error", auth_error.to_string()),
            );
            return Err(error);
        }

        // Step 3: Extract core metadata
        let issue_type = self.extract_issue_type(comment_body, comment_id, pr_number, author)?;
        let severity = self.extract_severity(comment_body, comment_id, pr_number, author)?;
        let description = self.extract_description(comment_body, comment_id, pr_number, author)?;

        // Step 4: Extract acceptance criteria
        let criteria_not_met =
            self.extract_criteria(comment_body, comment_id, pr_number, author)?;

        // Step 5: Extract optional sections
        let reproduction_steps = Self::extract_reproduction_steps_optional(comment_body);
        let (expected_behavior, actual_behavior) =
            Self::extract_expected_actual_optional(comment_body);

        // Step 6: Build metadata
        let metadata = FeedbackMetadata {
            author: author.to_string(),
            timestamp: Utc::now(),
            comment_id,
            pr_number,
            task_id: task_id.to_string(),
        };

        // Step 7: Construct final feedback structure
        let feedback = StructuredFeedback {
            issue_type,
            severity,
            description,
            criteria_not_met,
            reproduction_steps,
            expected_behavior,
            actual_behavior,
            metadata,
        };

        let duration = start_time.elapsed();
        if self.detailed_logging {
            info!(
                "Successfully parsed feedback for comment {} in {:?} - Issue: {:?}, Severity: {:?}, Criteria: {}",
                comment_id,
                duration,
                feedback.issue_type,
                feedback.severity,
                feedback.criteria_not_met.len()
            );
        }

        Ok(feedback)
    }

    /// Parse comment with enhanced error context
    pub fn parse_comment_with_context(
        &self,
        comment_body: &str,
        author: &str,
        comment_id: u64,
        pr_number: u32,
        task_id: &str,
    ) -> Result<StructuredFeedback> {
        match self.parse_comment(comment_body, author, comment_id, pr_number, task_id) {
            Ok(feedback) => Ok(feedback),
            Err(parse_error) => {
                let context = ErrorContext::new(parse_error.clone())
                    .with_comment(comment_id, pr_number, author.to_string())
                    .with_info("comment_length", comment_body.len().to_string())
                    .with_info(
                        "has_actionable_marker",
                        PatternExtractor::is_actionable_feedback(comment_body).to_string(),
                    );

                context.log();

                // Convert to anyhow error for external usage
                Err(anyhow::anyhow!(
                    "Feedback parsing failed: {}",
                    parse_error.user_message()
                ))
                .context(parse_error.suggested_action())
            }
        }
    }

    /// Quick validation - check if comment is parsable without full extraction
    pub fn validate_comment(&self, comment_body: &str, author: &str) -> ParseResult<()> {
        // Check size
        self.validate_comment_size(comment_body)?;

        // Check actionable
        if !PatternExtractor::is_actionable_feedback(comment_body) {
            return Err(ParseError::NotActionableFeedback);
        }

        // Check author
        self.author_validator.validate_author(author).map_err(|_| {
            ParseError::UnauthorizedAuthor {
                author: author.to_string(),
            }
        })?;

        // Quick checks for required fields
        if PatternExtractor::extract_issue_type_optional(comment_body).is_none() {
            return Err(ParseError::MissingRequiredField {
                field: "issue_type".to_string(),
            });
        }

        if PatternExtractor::extract_severity_optional(comment_body).is_none() {
            return Err(ParseError::MissingRequiredField {
                field: "severity".to_string(),
            });
        }

        if PatternExtractor::extract_description_optional(comment_body).is_none() {
            return Err(ParseError::MissingRequiredField {
                field: "description".to_string(),
            });
        }

        Ok(())
    }

    /// Extract issue type with error handling
    fn extract_issue_type(
        &self,
        body: &str,
        comment_id: u64,
        pr_number: u32,
        author: &str,
    ) -> ParseResult<IssueType> {
        PatternExtractor::extract_issue_type(body).map_err(|e| {
            let error = ParseError::IssueTypeError {
                details: e.to_string(),
            };
            self.log_error(
                &ErrorContext::new(error.clone())
                    .with_comment(comment_id, pr_number, author.to_string())
                    .with_info("extraction_error", e.to_string()),
            );
            error
        })
    }

    /// Extract severity with error handling
    fn extract_severity(
        &self,
        body: &str,
        comment_id: u64,
        pr_number: u32,
        author: &str,
    ) -> ParseResult<Severity> {
        PatternExtractor::extract_severity(body).map_err(|e| {
            let error = ParseError::SeverityError {
                details: e.to_string(),
            };
            self.log_error(
                &ErrorContext::new(error.clone())
                    .with_comment(comment_id, pr_number, author.to_string())
                    .with_info("extraction_error", e.to_string()),
            );
            error
        })
    }

    /// Extract description with error handling
    fn extract_description(
        &self,
        body: &str,
        comment_id: u64,
        pr_number: u32,
        author: &str,
    ) -> ParseResult<String> {
        PatternExtractor::extract_description(body).map_err(|e| {
            let error = ParseError::DescriptionError {
                details: e.to_string(),
            };
            self.log_error(
                &ErrorContext::new(error.clone())
                    .with_comment(comment_id, pr_number, author.to_string())
                    .with_info("extraction_error", e.to_string()),
            );
            error
        })
    }

    /// Extract criteria with error handling
    fn extract_criteria(
        &self,
        body: &str,
        comment_id: u64,
        pr_number: u32,
        author: &str,
    ) -> ParseResult<Vec<CriteriaStatus>> {
        MarkdownParser::extract_criteria_checkboxes(body).map_err(|e| {
            let error = ParseError::MarkdownParseError {
                details: e.to_string(),
            };
            self.log_error(
                &ErrorContext::new(error.clone())
                    .with_comment(comment_id, pr_number, author.to_string())
                    .with_info("extraction_error", e.to_string()),
            );
            error
        })
    }

    /// Extract reproduction steps (optional)
    fn extract_reproduction_steps_optional(body: &str) -> Option<Vec<String>> {
        PatternExtractor::extract_reproduction_steps_optional(body)
    }

    /// Extract expected/actual behavior (optional)
    fn extract_expected_actual_optional(body: &str) -> (Option<String>, Option<String>) {
        PatternExtractor::extract_expected_actual(body)
    }

    /// Validate comment size
    fn validate_comment_size(&self, comment_body: &str) -> ParseResult<()> {
        let size = comment_body.len();
        if size > self.max_comment_size {
            return Err(ParseError::ResourceExhausted {
                resource: "comment_size".to_string(),
                details: format!(
                    "Comment size {} exceeds maximum {}",
                    size, self.max_comment_size
                ),
            });
        }
        Ok(())
    }

    /// Log error with context
    fn log_error(&self, context: &ErrorContext) {
        if self.detailed_logging {
            context.log();
        }
    }

    /// Get author validator for configuration
    #[must_use]
    pub fn author_validator(&self) -> &AuthorValidator {
        &self.author_validator
    }

    /// Get mutable author validator for configuration
    pub fn author_validator_mut(&mut self) -> &mut AuthorValidator {
        &mut self.author_validator
    }

    /// Set detailed logging
    pub fn set_detailed_logging(&mut self, enabled: bool) {
        self.detailed_logging = enabled;
    }

    /// Set maximum comment size
    pub fn set_max_comment_size(&mut self, size: usize) {
        self.max_comment_size = size;
    }

    /// Get current configuration
    #[must_use]
    pub fn config(&self) -> ParserConfig {
        ParserConfig {
            detailed_logging: self.detailed_logging,
            max_comment_size: self.max_comment_size,
        }
    }
}

/// Configuration for the feedback parser
#[derive(Debug, Clone)]
pub struct ParserConfig {
    /// Whether to enable detailed logging
    pub detailed_logging: bool,
    /// Maximum comment size to process (in bytes)
    pub max_comment_size: usize,
}

impl Default for FeedbackParser {
    fn default() -> Self {
        Self::new()
    }
}

/// Convenience function for parsing feedback (equivalent to `parse_feedback_comment` from mod.rs)
pub fn parse_feedback_comment(
    comment_body: &str,
    author: &str,
    comment_id: u64,
    pr_number: u32,
    task_id: &str,
) -> Result<StructuredFeedback> {
    let parser = FeedbackParser::new();
    parser.parse_comment_with_context(comment_body, author, comment_id, pr_number, task_id)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::remediation::types::{IssueType, Severity};

    const SAMPLE_COMMENT: &str = r"🔴 Required Changes
**Issue Type**: [Bug]
**Severity**: [High]

### Description
The login button is not working properly when users click it.

### Acceptance Criteria Not Met
- [ ] User authentication works properly
- [x] Password reset functionality is implemented
- [ ] Error messages are user-friendly

### Steps to Reproduce
1. Navigate to login page
2. Enter valid credentials
3. Click login button

### Expected vs Actual
- **Expected**: User should be logged in and redirected to dashboard
- **Actual**: Page refreshes without login attempt";

    #[test]
    fn test_parse_complete_feedback() {
        let parser = FeedbackParser::new();
        let result = parser.parse_comment(SAMPLE_COMMENT, "5DLabs-Tess", 12345, 678, "task-2");

        assert!(result.is_ok());
        let feedback = result.unwrap();

        assert_eq!(feedback.issue_type, IssueType::Bug);
        assert_eq!(feedback.severity, Severity::High);
        assert!(feedback.description.contains("login button"));
        assert_eq!(feedback.criteria_not_met.len(), 3);
        assert!(feedback.reproduction_steps.is_some());
        assert!(feedback.expected_behavior.is_some());
        assert!(feedback.actual_behavior.is_some());

        assert_eq!(feedback.metadata.author, "5DLabs-Tess");
        assert_eq!(feedback.metadata.comment_id, 12345);
        assert_eq!(feedback.metadata.pr_number, 678);
        assert_eq!(feedback.metadata.task_id, "task-2");
    }

    #[test]
    fn test_parse_non_actionable_feedback() {
        let parser = FeedbackParser::new();
        let result = parser.parse_comment(
            "Just a regular comment",
            "5DLabs-Tess",
            12345,
            678,
            "task-2",
        );

        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            ParseError::NotActionableFeedback
        ));
    }

    #[test]
    fn test_parse_unauthorized_author() {
        let parser = FeedbackParser::new();
        let result =
            parser.parse_comment(SAMPLE_COMMENT, "unauthorized-user", 12345, 678, "task-2");

        assert!(result.is_err());
        if let ParseError::UnauthorizedAuthor { author } = result.unwrap_err() {
            assert_eq!(author, "unauthorized-user");
        } else {
            panic!("Expected UnauthorizedAuthor error");
        }
    }

    #[test]
    fn test_validate_comment() {
        let parser = FeedbackParser::new();

        // Valid comment
        assert!(parser
            .validate_comment(SAMPLE_COMMENT, "5DLabs-Tess")
            .is_ok());

        // Non-actionable
        assert!(parser
            .validate_comment("regular comment", "5DLabs-Tess")
            .is_err());

        // Unauthorized author
        assert!(parser.validate_comment(SAMPLE_COMMENT, "bad-user").is_err());
    }

    #[test]
    fn test_comment_size_validation() {
        let mut parser = FeedbackParser::new();
        parser.set_max_comment_size(100);

        let large_comment = "x".repeat(200);
        let result = parser.parse_comment(&large_comment, "5DLabs-Tess", 12345, 678, "task-2");

        assert!(result.is_err());
        if let ParseError::ResourceExhausted { resource, .. } = result.unwrap_err() {
            assert_eq!(resource, "comment_size");
        } else {
            panic!("Expected ResourceExhausted error");
        }
    }

    #[test]
    fn test_parse_minimal_feedback() {
        let minimal_comment = r"🔴 Required Changes
**Issue Type**: [Bug]
**Severity**: [Low]

### Description
Minimal test case.

### Acceptance Criteria Not Met
- [ ] Test criterion";

        let parser = FeedbackParser::new();
        let result = parser.parse_comment(minimal_comment, "5DLabs-Tess", 12345, 678, "task-2");

        assert!(result.is_ok());
        let feedback = result.unwrap();

        assert_eq!(feedback.issue_type, IssueType::Bug);
        assert_eq!(feedback.severity, Severity::Low);
        assert_eq!(feedback.criteria_not_met.len(), 1);
        assert!(feedback.reproduction_steps.is_none());
        assert!(feedback.expected_behavior.is_none());
        assert!(feedback.actual_behavior.is_none());
    }

    #[test]
    fn test_parser_configuration() {
        let mut parser = FeedbackParser::new();

        parser.set_detailed_logging(false);
        parser.set_max_comment_size(5 * 1024 * 1024); // 5MB

        let config = parser.config();
        assert!(!config.detailed_logging);
        assert_eq!(config.max_comment_size, 5 * 1024 * 1024);
    }

    #[test]
    fn test_convenience_function() {
        let result = parse_feedback_comment(SAMPLE_COMMENT, "5DLabs-Tess", 12345, 678, "task-2");

        assert!(result.is_ok());
        let feedback = result.unwrap();
        assert_eq!(feedback.issue_type, IssueType::Bug);
    }
}
