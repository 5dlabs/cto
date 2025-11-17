//! # Feedback Comment Parser Module
//!
//! This module provides comprehensive parsing functionality for QA feedback comments
//! in the Agent Remediation Loop system. It extracts structured data from GitHub PR
//! comments that follow the "🔴 Required Changes" format.
//!
//! ## Architecture
//!
//! The module is composed of several key components:
//!
//! - **Data Structures** (`types.rs`): Core types for representing structured feedback
//! - **Pattern Extraction** (`patterns.rs`): Regex-based extraction of metadata fields
//! - **Markdown Parser** (`markdown.rs`): Parsing of acceptance criteria checkboxes
//! - **Author Validation** (`auth.rs`): Authorization and caching for comment authors
//! - **Error Handling** (`error.rs`): Comprehensive error types and context
//! - **Main Parser** (`parser.rs`): Orchestration of all parsing components
//!
//! ## Usage
//!
//! ### Basic Usage
//!
//! ```rust
//! // Using the convenience function
//! let comment_body = "# Issue: Bug Report\n\n## Description\nSomething is broken";
//! let result = controller::remediation::parse_feedback_comment(
//!     comment_body,
//!     "5DLabs-Tess",
//!     12345,
//!     678,
//!     "task-2"
//! );
//!
//! match result {
//!     Ok(feedback) => {
//!         println!("Issue Type: {:?}", feedback.issue_type);
//!         println!("Severity: {:?}", feedback.severity);
//!         println!("Criteria to fix: {}", feedback.criteria_not_met.len());
//!     }
//!     Err(e) => {
//!         eprintln!("Parsing failed: {}", e);
//!     }
//! }
//! ```
//!
//! ### Advanced Usage with Custom Configuration
//!
//! ```rust,ignore
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! let mut validator = controller::remediation::AuthorValidator::new();
//! validator.add_approved_author("custom-reviewer")
//!     .expect("Failed to add author");
//!
//! let parser = controller::remediation::FeedbackParser::with_validator(validator);
//!
//! let comment_body = "# Issue: Bug Report\n\n## Description\nSomething is broken";
//! let author = "custom-reviewer";
//! let comment_id = 12345;
//! let pr_number = 678;
//! let task_id = "task-2";
//!
//! let feedback = parser.parse_comment(
//!     comment_body,
//!     author,
//!     comment_id,
//!     pr_number,
//!     task_id
//! )?;
//! # Ok(())
//! # }
//! ```
//!
//! ## Expected Comment Format
//!
//! The parser expects comments in the following structured format:
//!
//! ```markdown
//! 🔴 Required Changes
//! **Issue Type**: [Bug|Missing Feature|Regression|Performance]
//! **Severity**: [Critical|High|Medium|Low]
//!
//! ### Description
//! [Clear description of the issue]
//!
//! ### Acceptance Criteria Not Met
//! - [ ] Specific criterion not satisfied
//! - [ ] Another missing requirement
//! - [x] This criterion was already met
//!
//! ### Steps to Reproduce (optional)
//! 1. Step one
//! 2. Step two
//! 3. Step three
//!
//! ### Expected vs Actual (optional)
//! - **Expected**: [what should happen]
//! - **Actual**: [what actually happens]
//! ```
//!
//! ## Error Handling
//!
//! The module provides comprehensive error handling with specific error types:
//!
//! - `NotActionableFeedback`: Missing "🔴 Required Changes" marker
//! - `UnauthorizedAuthor`: Author not in approved list
//! - `MissingRequiredField`: Required field (Issue Type, Severity, Description) missing
//! - `InvalidFieldValue`: Field value doesn't match expected format
//! - `NoCriteriaFound`: No acceptance criteria checkboxes found
//! - `ParseError`: Generic parsing failures with detailed context
//!
//! ## Performance Considerations
//!
//! - Regex patterns are compiled once using `lazy_static`
//! - Author validation includes caching with configurable TTL (default: 5 minutes)
//! - Memory usage is bounded with configurable maximum comment size
//! - Concurrent parsing operations are supported
//!
//! ## Security Features
//!
//! - XSS prevention: HTML/script tags are not executed
//! - Command injection prevention: Shell metacharacters are safe
//! - `ReDoS` prevention: Regex patterns have complexity bounds
//! - Authorization bypass prevention: Cache poisoning is prevented
//! - Memory exhaustion prevention: Large inputs don't cause OOM

// Public module exports
pub mod auth;
pub mod error;
pub mod markdown;
pub mod parser;
pub mod patterns;
pub mod state;
pub mod types;

// Re-export key types for convenience
pub use types::{
    CriteriaStatus, FeedbackMetadata, IssueType, OptionalBehavior, OptionalSteps, Severity,
    StructuredFeedback,
};

pub use parser::{parse_feedback_comment, FeedbackParser, ParserConfig};

pub use error::{ErrorContext, ParseError, ParseResult};

pub use auth::{AuthorValidator, SharedAuthorValidator};

pub use markdown::MarkdownParser;
pub use patterns::PatternExtractor;
pub use state::{
    RemediationState, RemediationStateManager, RemediationStatistics, RemediationStatus,
};

/// Create a new feedback parser with default configuration
///
/// This is a convenience function for creating a parser with standard settings.
/// For custom configuration, use `FeedbackParser::with_config()` directly.
#[must_use]
pub fn new_parser() -> FeedbackParser {
    FeedbackParser::new()
}

/// Create a parser with custom author validator
///
/// Useful when you need specific authorization rules or additional approved authors.
#[must_use]
pub fn parser_with_validator(validator: AuthorValidator) -> FeedbackParser {
    FeedbackParser::with_validator(validator)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::remediation::types::{IssueType, Severity};

    const TEST_COMMENT: &str = r"🔴 Required Changes
**Issue Type**: [Bug]
**Severity**: [High]

### Description
Test feedback for parsing functionality.

### Acceptance Criteria Not Met
- [ ] Authentication works
- [x] UI renders correctly";

    #[test]
    fn test_public_api() {
        let result = parse_feedback_comment(TEST_COMMENT, "5DLabs-Tess", 12345, 678, "task-2");

        assert!(result.is_ok());
        let feedback = result.unwrap();

        assert_eq!(feedback.issue_type, IssueType::Bug);
        assert_eq!(feedback.severity, Severity::High);
        assert!(feedback.description.contains("parsing functionality"));
        assert_eq!(feedback.criteria_not_met.len(), 2);
    }

    #[test]
    fn test_new_parser() {
        let parser = new_parser();

        let result = parser.parse_comment(TEST_COMMENT, "5DLabs-Tess", 12345, 678, "task-2");

        assert!(result.is_ok());
    }

    #[test]
    fn test_parser_with_custom_validator() {
        let mut validator = AuthorValidator::new();
        validator
            .add_approved_author("custom-user")
            .expect("Failed to add author");

        let parser = parser_with_validator(validator);

        let result = parser.parse_comment(TEST_COMMENT, "custom-user", 12345, 678, "task-2");

        assert!(result.is_ok());
    }

    #[test]
    fn test_module_exports() {
        // Test that all expected types are exported
        #[allow(clippy::no_effect_underscore_binding)]
        {
            let _issue_type: IssueType = IssueType::Bug;
            let _severity: Severity = Severity::High;
        }
        let _parser = FeedbackParser::new();
        let _validator = AuthorValidator::new();
    }
}
