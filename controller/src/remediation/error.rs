use thiserror::Error;

/// Comprehensive error types for feedback parsing operations
#[derive(Error, Debug, Clone)]
pub enum ParseError {
    /// Comment does not contain actionable feedback marker
    #[error("Comment is not actionable feedback - missing '🔴 Required Changes' marker")]
    NotActionableFeedback,

    /// Author is not authorized to provide feedback
    #[error("Author '{author}' is not authorized to provide feedback")]
    UnauthorizedAuthor { author: String },

    /// Required field is missing from the comment
    #[error("Required field '{field}' is missing from the comment")]
    MissingRequiredField { field: String },

    /// Invalid value provided for a field
    #[error("Invalid value '{value}' for field '{field}' - expected {expected}")]
    InvalidFieldValue {
        field: String,
        value: String,
        expected: String,
    },

    /// Malformed comment structure preventing parsing
    #[error("Malformed comment structure: {reason}")]
    MalformedComment { reason: String },

    /// No acceptance criteria checkboxes found
    #[error("No acceptance criteria checkboxes found in the comment")]
    NoCriteriaFound,

    /// All acceptance criteria appear to be met
    #[error("All acceptance criteria appear to be met - no remediation needed")]
    AllCriteriaMet,

    /// Issue type could not be extracted or parsed
    #[error("Issue type extraction failed: {details}")]
    IssueTypeError { details: String },

    /// Severity level could not be extracted or parsed
    #[error("Severity extraction failed: {details}")]
    SeverityError { details: String },

    /// Description section could not be extracted
    #[error("Description extraction failed: {details}")]
    DescriptionError { details: String },

    /// Reproduction steps could not be extracted
    #[error("Reproduction steps extraction failed: {details}")]
    ReproductionStepsError { details: String },

    /// Expected vs actual behavior could not be extracted
    #[error("Expected/actual behavior extraction failed: {details}")]
    ExpectedActualError { details: String },

    /// Markdown parsing failed
    #[error("Markdown parsing failed: {details}")]
    MarkdownParseError { details: String },

    /// Regex compilation or matching failed
    #[error("Regex operation failed: {details}")]
    RegexError { details: String },

    /// Author validation failed
    #[error("Author validation failed: {details}")]
    AuthorValidationError { details: String },

    /// Cache operation failed
    #[error("Cache operation failed: {details}")]
    CacheError { details: String },

    /// Serialization/deserialization failed
    #[error("Serialization error: {details}")]
    SerializationError { details: String },

    /// Generic parsing error with context
    #[error("Parsing error: {message}")]
    Generic { message: String },

    /// Network or external service error
    #[error("External service error: {service} - {details}")]
    ExternalServiceError { service: String, details: String },

    /// Configuration error
    #[error("Configuration error: {details}")]
    ConfigurationError { details: String },

    /// Timeout error
    #[error("Operation timed out: {operation}")]
    TimeoutError { operation: String },

    /// Memory or resource exhaustion
    #[error("Resource exhausted: {resource} - {details}")]
    ResourceExhausted { resource: String, details: String },
}

/// Type alias for `ParseResult`
pub type ParseResult<T> = Result<T, ParseError>;

impl ParseError {
    /// Check if error is recoverable (can be retried)
    #[must_use]
    pub fn is_recoverable(&self) -> bool {
        matches!(
            self,
            ParseError::ExternalServiceError { .. }
                | ParseError::TimeoutError { .. }
                | ParseError::CacheError { .. }
        )
    }

    /// Check if error indicates authorization failure
    #[must_use]
    pub fn is_authorization_error(&self) -> bool {
        matches!(
            self,
            ParseError::UnauthorizedAuthor { .. } | ParseError::AuthorValidationError { .. }
        )
    }

    /// Check if error indicates malformed input
    #[must_use]
    pub fn is_malformed_input(&self) -> bool {
        matches!(
            self,
            ParseError::MalformedComment { .. }
                | ParseError::InvalidFieldValue { .. }
                | ParseError::MarkdownParseError { .. }
                | ParseError::RegexError { .. }
        )
    }

    /// Check if error indicates missing required data
    #[must_use]
    pub fn is_missing_data(&self) -> bool {
        matches!(
            self,
            ParseError::MissingRequiredField { .. }
                | ParseError::NoCriteriaFound
                | ParseError::IssueTypeError { .. }
                | ParseError::SeverityError { .. }
                | ParseError::DescriptionError { .. }
        )
    }

    /// Get error category as string
    #[must_use]
    pub fn category(&self) -> &'static str {
        match self {
            ParseError::NotActionableFeedback => "not_actionable",
            ParseError::UnauthorizedAuthor { .. } => "authorization",
            ParseError::MissingRequiredField { .. } => "missing_data",
            ParseError::InvalidFieldValue { .. } => "invalid_data",
            ParseError::MalformedComment { .. } => "malformed_input",
            ParseError::NoCriteriaFound => "missing_data",
            ParseError::AllCriteriaMet => "no_action_needed",
            ParseError::IssueTypeError { .. } => "extraction_failure",
            ParseError::SeverityError { .. } => "extraction_failure",
            ParseError::DescriptionError { .. } => "extraction_failure",
            ParseError::ReproductionStepsError { .. } => "extraction_failure",
            ParseError::ExpectedActualError { .. } => "extraction_failure",
            ParseError::MarkdownParseError { .. } => "parsing_failure",
            ParseError::RegexError { .. } => "parsing_failure",
            ParseError::AuthorValidationError { .. } => "authorization",
            ParseError::CacheError { .. } => "system_error",
            ParseError::SerializationError { .. } => "system_error",
            ParseError::Generic { .. } => "generic_error",
            ParseError::ExternalServiceError { .. } => "external_error",
            ParseError::ConfigurationError { .. } => "configuration_error",
            ParseError::TimeoutError { .. } => "timeout_error",
            ParseError::ResourceExhausted { .. } => "resource_error",
        }
    }

    /// Convert error to user-friendly message
    #[must_use]
    pub fn user_message(&self) -> String {
        match self {
            ParseError::NotActionableFeedback => {
                "This comment doesn't contain actionable feedback. Make sure it includes '🔴 Required Changes'.".to_string()
            }
            ParseError::UnauthorizedAuthor { author } => {
                format!("Author '{author}' is not authorized to provide feedback. Contact an administrator.")
            }
            ParseError::MissingRequiredField { field } => {
                format!("Required field '{field}' is missing from the comment.")
            }
            ParseError::InvalidFieldValue { field, value, expected } => {
                format!("Invalid value '{value}' for field '{field}'. Expected: {expected}")
            }
            ParseError::MalformedComment { reason } => {
                format!("Comment structure is malformed: {reason}")
            }
            ParseError::NoCriteriaFound => {
                "No acceptance criteria checkboxes found. Make sure to include a checklist in the 'Acceptance Criteria Not Met' section.".to_string()
            }
            ParseError::AllCriteriaMet => {
                "All acceptance criteria appear to be met - no remediation needed.".to_string()
            }
            ParseError::IssueTypeError { .. } => {
                "Could not extract issue type. Make sure it follows the format: **Issue Type**: [Bug|Missing Feature|Regression|Performance]".to_string()
            }
            ParseError::SeverityError { .. } => {
                "Could not extract severity level. Make sure it follows the format: **Severity**: [Critical|High|Medium|Low]".to_string()
            }
            ParseError::DescriptionError { .. } => {
                "Could not extract description. Make sure to include a '### Description' section.".to_string()
            }
            ParseError::ReproductionStepsError { .. } => {
                "Could not extract reproduction steps. Make sure to include a '### Steps to Reproduce' section.".to_string()
            }
            ParseError::ExpectedActualError { .. } => {
                "Could not extract expected vs actual behavior.".to_string()
            }
            ParseError::MarkdownParseError { details } => {
                format!("Failed to parse markdown content: {details}")
            }
            ParseError::RegexError { details } => {
                format!("Pattern matching failed: {details}")
            }
            ParseError::AuthorValidationError { details } => {
                format!("Author validation failed: {details}")
            }
            ParseError::CacheError { details } => {
                format!("System cache error: {details}")
            }
            ParseError::SerializationError { details } => {
                format!("Data serialization error: {details}")
            }
            ParseError::Generic { message } => {
                format!("Parsing failed: {message}")
            }
            ParseError::ExternalServiceError { service, details } => {
                format!("External service '{service}' error: {details}")
            }
            ParseError::ConfigurationError { details } => {
                format!("Configuration error: {details}")
            }
            ParseError::TimeoutError { operation } => {
                format!("Operation '{operation}' timed out")
            }
            ParseError::ResourceExhausted { resource, details } => {
                format!("Resource '{resource}' exhausted: {details}")
            }
        }
    }

    /// Get suggested remediation action
    #[must_use]
    pub fn suggested_action(&self) -> &'static str {
        match self {
            ParseError::NotActionableFeedback => {
                "Add '🔴 Required Changes' marker to make this actionable feedback"
            }
            ParseError::UnauthorizedAuthor { .. } => {
                "Contact administrator to be added to approved reviewers list"
            }
            ParseError::MissingRequiredField { .. } => {
                "Add the missing required field to your comment"
            }
            ParseError::InvalidFieldValue { .. } => {
                "Correct the field value to match expected format"
            }
            ParseError::MalformedComment { .. } => "Fix the comment structure and formatting",
            ParseError::NoCriteriaFound => "Add acceptance criteria checkboxes in proper format",
            ParseError::AllCriteriaMet => "Review if remediation is actually needed",
            ParseError::IssueTypeError { .. } => {
                "Use correct issue type format with square brackets"
            }
            ParseError::SeverityError { .. } => "Use correct severity format with square brackets",
            ParseError::DescriptionError { .. } => {
                "Add ### Description section with clear issue description"
            }
            ParseError::ReproductionStepsError { .. } => {
                "Add ### Steps to Reproduce section with numbered steps"
            }
            ParseError::ExpectedActualError { .. } => "Add ### Expected vs Actual section",
            ParseError::MarkdownParseError { .. } => "Fix markdown syntax and formatting",
            ParseError::RegexError { .. } => "Report to system administrator",
            ParseError::AuthorValidationError { .. } => "Check author permissions and try again",
            ParseError::CacheError { .. } => "Try again in a few moments",
            ParseError::SerializationError { .. } => "Report to system administrator",
            ParseError::Generic { .. } => "Check comment format and try again",
            ParseError::ExternalServiceError { .. } => "Try again later or contact administrator",
            ParseError::ConfigurationError { .. } => "Report to system administrator",
            ParseError::TimeoutError { .. } => "Try again with smaller input",
            ParseError::ResourceExhausted { .. } => "Try again later with smaller input",
        }
    }
}

/// Error context wrapper for adding additional information
#[derive(Debug)]
pub struct ErrorContext {
    pub error: ParseError,
    pub comment_id: Option<u64>,
    pub pr_number: Option<u32>,
    pub author: Option<String>,
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub additional_info: std::collections::HashMap<String, String>,
}

impl ErrorContext {
    /// Create new error context
    #[must_use]
    pub fn new(error: ParseError) -> Self {
        Self {
            error,
            comment_id: None,
            pr_number: None,
            author: None,
            timestamp: chrono::Utc::now(),
            additional_info: std::collections::HashMap::new(),
        }
    }

    /// Add comment context
    #[must_use]
    pub fn with_comment(mut self, comment_id: u64, pr_number: u32, author: String) -> Self {
        self.comment_id = Some(comment_id);
        self.pr_number = Some(pr_number);
        self.author = Some(author);
        self
    }

    /// Add additional context information
    #[must_use]
    pub fn with_info(mut self, key: &str, value: String) -> Self {
        self.additional_info.insert(key.to_string(), value);
        self
    }

    /// Get user-friendly error message with context
    #[must_use]
    pub fn user_message(&self) -> String {
        let mut message = self.error.user_message();

        if let Some(pr_number) = self.pr_number {
            message.push_str(&format!(" (PR #{pr_number})"));
        }

        if let Some(author) = self.author.as_ref() {
            message.push_str(&format!(" (Author: {author})"));
        }

        message
    }

    /// Log error with full context
    pub fn log(&self) {
        let category = self.error.category();
        let message = format!(
            "Parse error [{}] - {} - Suggested action: {}",
            category,
            self.error,
            self.error.suggested_action()
        );

        match category {
            "authorization" | "external_error" | "resource_error" => {
                tracing::warn!("{}", message);
            }
            "missing_data" | "invalid_data" | "malformed_input" => {
                tracing::info!("{}", message);
            }
            _ => {
                tracing::error!("{}", message);
            }
        }

        // Log additional context
        if let Some(comment_id) = self.comment_id {
            tracing::debug!("Comment ID: {}", comment_id);
        }
        if let Some(pr_number) = self.pr_number {
            tracing::debug!("PR Number: {}", pr_number);
        }
        if let Some(author) = self.author.as_ref() {
            tracing::debug!("Author: {}", author);
        }

        for (key, value) in &self.additional_info {
            tracing::debug!("{}: {}", key, value);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_error_display() {
        let error = ParseError::NotActionableFeedback;
        assert!(error.to_string().contains("not actionable feedback"));

        let error = ParseError::UnauthorizedAuthor {
            author: "test-user".to_string(),
        };
        assert!(error.to_string().contains("test-user"));
        assert!(error.to_string().contains("not authorized"));

        let error = ParseError::MissingRequiredField {
            field: "description".to_string(),
        };
        assert!(error.to_string().contains("description"));
        assert!(error.to_string().contains("missing"));
    }

    #[test]
    fn test_error_categories() {
        assert_eq!(
            ParseError::NotActionableFeedback.category(),
            "not_actionable"
        );
        assert_eq!(
            ParseError::UnauthorizedAuthor {
                author: String::new()
            }
            .category(),
            "authorization"
        );
        assert_eq!(
            ParseError::MissingRequiredField {
                field: String::new()
            }
            .category(),
            "missing_data"
        );
        assert_eq!(
            ParseError::InvalidFieldValue {
                field: String::new(),
                value: String::new(),
                expected: String::new()
            }
            .category(),
            "invalid_data"
        );
        assert_eq!(
            ParseError::MalformedComment {
                reason: String::new()
            }
            .category(),
            "malformed_input"
        );
        assert_eq!(ParseError::NoCriteriaFound.category(), "missing_data");
        assert_eq!(ParseError::AllCriteriaMet.category(), "no_action_needed");
    }

    #[test]
    fn test_error_classification() {
        // Recoverable errors
        assert!(ParseError::ExternalServiceError {
            service: String::new(),
            details: String::new()
        }
        .is_recoverable());
        assert!(ParseError::TimeoutError {
            operation: String::new()
        }
        .is_recoverable());
        assert!(ParseError::CacheError {
            details: String::new()
        }
        .is_recoverable());

        // Non-recoverable errors
        assert!(!ParseError::NotActionableFeedback.is_recoverable());
        assert!(!ParseError::MalformedComment {
            reason: String::new()
        }
        .is_recoverable());

        // Authorization errors
        assert!(ParseError::UnauthorizedAuthor {
            author: String::new()
        }
        .is_authorization_error());
        assert!(ParseError::AuthorValidationError {
            details: String::new()
        }
        .is_authorization_error());

        // Malformed input errors
        assert!(ParseError::MalformedComment {
            reason: String::new()
        }
        .is_malformed_input());
        assert!(ParseError::InvalidFieldValue {
            field: String::new(),
            value: String::new(),
            expected: String::new()
        }
        .is_malformed_input());
        assert!(ParseError::RegexError {
            details: String::new()
        }
        .is_malformed_input());

        // Missing data errors
        assert!(ParseError::MissingRequiredField {
            field: String::new()
        }
        .is_missing_data());
        assert!(ParseError::NoCriteriaFound.is_missing_data());
        assert!(ParseError::IssueTypeError {
            details: String::new()
        }
        .is_missing_data());
    }

    #[test]
    fn test_user_messages() {
        let error = ParseError::NotActionableFeedback;
        let message = error.user_message();
        assert!(message.contains("🔴 Required Changes"));

        let error = ParseError::UnauthorizedAuthor {
            author: "bad-user".to_string(),
        };
        let message = error.user_message();
        assert!(message.contains("bad-user"));
        assert!(message.contains("administrator"));

        let error = ParseError::NoCriteriaFound;
        let message = error.user_message();
        assert!(message.contains("Acceptance Criteria"));
    }

    #[test]
    fn test_suggested_actions() {
        let error = ParseError::NotActionableFeedback;
        assert!(error.suggested_action().contains("🔴 Required Changes"));

        let error = ParseError::UnauthorizedAuthor {
            author: String::new(),
        };
        assert!(error.suggested_action().contains("administrator"));

        let error = ParseError::MissingRequiredField {
            field: "severity".to_string(),
        };
        assert!(error.suggested_action().contains("Add the missing"));
    }

    #[test]
    fn test_error_context() {
        let error = ParseError::UnauthorizedAuthor {
            author: "test-user".to_string(),
        };
        let context = ErrorContext::new(error)
            .with_comment(12345, 678, "test-author".to_string())
            .with_info("extra", "additional info".to_string());

        let message = context.user_message();
        assert!(message.contains("PR #678"));
        assert!(message.contains("Author: test-author"));
        assert!(message.contains("test-user"));

        assert_eq!(context.comment_id, Some(12345));
        assert_eq!(context.pr_number, Some(678));
        assert_eq!(context.author, Some("test-author".to_string()));
        assert_eq!(
            context.additional_info.get("extra"),
            Some(&"additional info".to_string())
        );
    }

    #[test]
    fn test_error_context_without_comment_info() {
        let error = ParseError::NotActionableFeedback;
        let context = ErrorContext::new(error);

        let message = context.user_message();
        assert!(!message.contains("PR #"));
        assert!(!message.contains("Author:"));

        assert!(context.comment_id.is_none());
        assert!(context.pr_number.is_none());
        assert!(context.author.is_none());
    }
}
