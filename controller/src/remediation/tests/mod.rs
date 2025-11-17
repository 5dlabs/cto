//! Comprehensive test suite for the feedback comment parser

mod fixtures;

use std::sync::Arc;
use crate::remediation::{
    types::{StructuredFeedback, IssueType, Severity, CriteriaStatus},
    parser::FeedbackParser,
    auth::AuthorValidator,
    error::{ParseError, ParseResult},
    patterns::PatternExtractor,
    markdown::MarkdownParser,
};

use fixtures::*;

/// Test complete feedback parsing workflow
#[cfg(test)]
mod integration_tests {
    use super::*;

    #[test]
    fn test_parse_complete_feedback() {
        let parser = FeedbackParser::new();
        let result = parser.parse_comment(
            COMPLETE_FEEDBACK_COMMENT,
            "5DLabs-Tess",
            12345,
            678,
            "task-2",
        );

        assert!(result.is_ok());
        let feedback = result.unwrap();

        // Verify core metadata
        assert_eq!(feedback.issue_type, IssueType::Bug);
        assert_eq!(feedback.severity, Severity::High);
        assert!(feedback.description.contains("login button"));

        // Verify criteria extraction
        assert_eq!(feedback.criteria_not_met.len(), 4);
        let unmet_criteria: Vec<_> = feedback.criteria_not_met
            .iter()
            .filter(|c| !c.completed)
            .collect();
        assert_eq!(unmet_criteria.len(), 3);

        // Verify optional sections
        assert!(feedback.reproduction_steps.is_some());
        assert_eq!(feedback.reproduction_steps.as_ref().unwrap().len(), 5);
        assert!(feedback.expected_behavior.is_some());
        assert!(feedback.actual_behavior.is_some());
    }

    #[test]
    fn test_parse_minimal_feedback() {
        let parser = FeedbackParser::new();
        let result = parser.parse_comment(
            MINIMAL_FEEDBACK_COMMENT,
            "5DLabs-Tess",
            54321,
            999,
            "task-test",
        );

        assert!(result.is_ok());
        let feedback = result.unwrap();

        assert_eq!(feedback.issue_type, IssueType::Bug);
        assert_eq!(feedback.severity, Severity::Low);
        assert_eq!(feedback.criteria_not_met.len(), 1);
        assert!(feedback.reproduction_steps.is_none());
    }

    #[test]
    fn test_parse_non_actionable_feedback() {
        let parser = FeedbackParser::new();
        let result = parser.parse_comment(
            NON_ACTIONABLE_COMMENT,
            "5DLabs-Tess",
            12345,
            678,
            "task-2",
        );

        assert!(result.is_err());
        if let ParseError::NotActionableFeedback = result.unwrap_err() {
            // Expected error
        } else {
            panic!("Expected NotActionableFeedback error");
        }
    }

    #[test]
    fn test_parse_unauthorized_author() {
        let parser = FeedbackParser::new();
        let result = parser.parse_comment(
            AUTHORIZED_AUTHOR_COMMENT,
            "unauthorized-user",
            12345,
            678,
            "task-2",
        );

        assert!(result.is_err());
        if let ParseError::UnauthorizedAuthor { author } = result.unwrap_err() {
            assert_eq!(author, "unauthorized-user");
        } else {
            panic!("Expected UnauthorizedAuthor error");
        }
    }

    #[test]
    fn test_parse_malformed_comment() {
        let parser = FeedbackParser::new();
        let result = parser.parse_comment(
            MALFORMED_COMMENT,
            "5DLabs-Tess",
            12345,
            678,
            "task-2",
        );

        // Should handle gracefully - may fail on severity parsing
        assert!(result.is_err());
    }
}

/// Test individual components
#[cfg(test)]
mod component_tests {
    use super::*;

    #[test]
    fn test_issue_type_extraction() {
        assert!(PatternExtractor::extract_issue_type(BUG_FEEDBACK).is_ok());
        assert!(PatternExtractor::extract_issue_type(FEATURE_FEEDBACK).is_ok());
        assert!(PatternExtractor::extract_issue_type(REGRESSION_FEEDBACK).is_ok());
        assert!(PatternExtractor::extract_issue_type(PERFORMANCE_FEEDBACK).is_ok());

        assert_eq!(PatternExtractor::extract_issue_type(BUG_FEEDBACK).unwrap(), IssueType::Bug);
        assert_eq!(PatternExtractor::extract_issue_type(FEATURE_FEEDBACK).unwrap(), IssueType::MissingFeature);
    }

    #[test]
    fn test_severity_extraction() {
        assert!(PatternExtractor::extract_severity(CRITICAL_SEVERITY).is_ok());
        assert!(PatternExtractor::extract_severity(HIGH_SEVERITY).is_ok());
        assert!(PatternExtractor::extract_severity(MEDIUM_SEVERITY).is_ok());
        assert!(PatternExtractor::extract_severity(LOW_SEVERITY).is_ok());

        assert_eq!(PatternExtractor::extract_severity(CRITICAL_SEVERITY).unwrap(), Severity::Critical);
        assert_eq!(PatternExtractor::extract_severity(HIGH_SEVERITY).unwrap(), Severity::High);
    }

    #[test]
    fn test_criteria_parsing() {
        let result = MarkdownParser::extract_criteria_checkboxes(COMPLETE_FEEDBACK_COMMENT);
        assert!(result.is_ok());

        let criteria = result.unwrap();
        assert_eq!(criteria.len(), 4);

        // Check completion status
        assert!(!criteria[0].completed); // User authentication
        assert!(!criteria[1].completed); // Error messages
        assert!(criteria[2].completed);  // Login form renders
        assert!(!criteria[3].completed); // Password reset
    }

    #[test]
    fn test_author_validation() {
        let validator = AuthorValidator::new();

        assert!(validator.validate_author("5DLabs-Tess").is_ok());
        assert!(validator.validate_author("5DLabs-Developer").is_ok());
        assert!(validator.validate_author("random-user").is_err());
    }

    #[test]
    fn test_pattern_extraction_edge_cases() {
        // Test missing sections
        assert!(PatternExtractor::extract_issue_type(MISSING_ISSUE_TYPE).is_err());
        assert!(PatternExtractor::extract_severity(MISSING_SEVERITY).is_err());
        assert!(PatternExtractor::extract_description(MISSING_DESCRIPTION).is_err());
    }
}

/// Test error handling scenarios
#[cfg(test)]
mod error_tests {
    use super::*;

    #[test]
    fn test_missing_required_fields() {
        let parser = FeedbackParser::new();

        // Missing issue type
        let result = parser.parse_comment(
            MISSING_ISSUE_TYPE,
            "5DLabs-Tess",
            12345,
            678,
            "task-2",
        );
        assert!(result.is_err());

        // Missing severity
        let result = parser.parse_comment(
            MISSING_SEVERITY,
            "5DLabs-Tess",
            12345,
            678,
            "task-2",
        );
        assert!(result.is_err());

        // Missing description
        let result = parser.parse_comment(
            MISSING_DESCRIPTION,
            "5DLabs-Tess",
            12345,
            678,
            "task-2",
        );
        assert!(result.is_err());
    }

    #[test]
    fn test_missing_criteria_section() {
        let parser = FeedbackParser::new();
        let result = parser.parse_comment(
            MISSING_CRITERIA,
            "5DLabs-Tess",
            12345,
            678,
            "task-2",
        );
        assert!(result.is_err());
    }

    #[test]
    fn test_error_context_and_logging() {
        let parser = FeedbackParser::new();
        let result = parser.parse_comment_with_context(
            NON_ACTIONABLE_COMMENT,
            "5DLabs-Tess",
            12345,
            678,
            "task-2",
        );

        assert!(result.is_err());
        let error_msg = result.unwrap_err().to_string();
        assert!(error_msg.contains("not actionable"));
    }
}

/// Test edge cases and special scenarios
#[cfg(test)]
mod edge_case_tests {
    use super::*;

    #[test]
    fn test_unicode_handling() {
        let parser = FeedbackParser::new();
        let result = parser.parse_comment(
            UNICODE_COMMENT,
            "5DLabs-Tess",
            12345,
            678,
            "task-2",
        );

        assert!(result.is_ok());
        let feedback = result.unwrap();
        assert!(feedback.description.contains("🚀"));
        assert!(feedback.description.contains("αβγδε"));
    }

    #[test]
    fn test_whitespace_handling() {
        let parser = FeedbackParser::new();
        let result = parser.parse_comment(
            WHITESPACE_COMMENT,
            "5DLabs-Tess",
            12345,
            678,
            "task-2",
        );

        assert!(result.is_ok());
        let feedback = result.unwrap();
        // Should handle extra whitespace gracefully
        assert!(feedback.criteria_not_met.len() >= 1);
    }

    #[test]
    fn test_empty_sections() {
        let parser = FeedbackParser::new();
        let result = parser.parse_comment(
            EMPTY_SECTIONS,
            "5DLabs-Tess",
            12345,
            678,
            "task-2",
        );

        // Should handle empty sections gracefully
        assert!(result.is_ok());
        let feedback = result.unwrap();
        assert!(feedback.reproduction_steps.is_none());
        assert!(feedback.expected_behavior.is_none());
    }

    #[test]
    fn test_alternative_checkbox_formats() {
        let result = MarkdownParser::extract_criteria_checkboxes(ALTERNATIVE_CHECKBOX_FORMAT);
        assert!(result.is_ok());

        let criteria = result.unwrap();
        assert!(criteria.len() >= 3); // Should handle different formats
    }
}

/// Test performance characteristics
#[cfg(test)]
mod performance_tests {
    use super::*;
    use std::time::Instant;

    #[test]
    fn test_typical_comment_performance() {
        let parser = FeedbackParser::new();
        let start = Instant::now();

        let result = parser.parse_comment(
            COMPLETE_FEEDBACK_COMMENT,
            "5DLabs-Tess",
            12345,
            678,
            "task-2",
        );

        let duration = start.elapsed();
        assert!(result.is_ok());
        assert!(duration.as_millis() < 10, "Parsing took {}ms, expected < 10ms", duration.as_millis());
    }

    #[test]
    fn test_large_comment_performance() {
        let parser = FeedbackParser::new();
        let large_comment = create_large_comment(100); // 100KB
        let start = Instant::now();

        let result = parser.parse_comment(
            &large_comment,
            "5DLabs-Tess",
            12345,
            678,
            "task-2",
        );

        let duration = start.elapsed();
        assert!(result.is_ok());
        assert!(duration.as_millis() < 100, "Large comment parsing took {}ms, expected < 100ms", duration.as_millis());
    }

    #[test]
    fn test_comment_size_limits() {
        let mut parser = FeedbackParser::new();
        parser.set_max_comment_size(1000); // 1KB limit

        let large_comment = create_large_comment(2); // 2KB comment
        let result = parser.parse_comment(
            &large_comment,
            "5DLabs-Tess",
            12345,
            678,
            "task-2",
        );

        assert!(result.is_err());
        if let ParseError::ResourceExhausted { resource, .. } = result.unwrap_err() {
            assert_eq!(resource, "comment_size");
        } else {
            panic!("Expected ResourceExhausted error");
        }
    }
}

/// Test author validation caching and team management
#[cfg(test)]
mod author_validation_tests {
    use super::*;

    #[test]
    fn test_author_cache() {
        let validator = AuthorValidator::new();

        // First validation
        assert!(validator.validate_author("5DLabs-Tess").is_ok());
        assert_eq!(validator.auth_cache.len(), 1);

        // Second validation (should use cache)
        assert!(validator.validate_author("5DLabs-Tess").is_ok());
        assert_eq!(validator.auth_cache.len(), 1); // Still 1 entry
    }

    #[test]
    fn test_team_prefix_validation() {
        let mut validator = AuthorValidator::new();
        validator.add_team_prefix("MyTeam-").unwrap();

        assert!(validator.validate_author("MyTeam-Dev").is_ok());
        assert!(validator.validate_author("MyTeam-QA").is_ok());
        assert!(validator.validate_author("OtherTeam-Dev").is_err());
    }

    #[test]
    fn test_dynamic_author_management() {
        let mut validator = AuthorValidator::new();

        // Add author
        validator.add_approved_author("new-reviewer").unwrap();
        assert!(validator.validate_author("new-reviewer").is_ok());

        // Remove author
        validator.remove_approved_author("new-reviewer").unwrap();
        assert!(validator.validate_author("new-reviewer").is_err());
    }

    #[test]
    fn test_cache_invalidation() {
        let validator = AuthorValidator::new();

        // Add to cache
        let _ = validator.validate_author("5DLabs-Tess");
        assert_eq!(validator.auth_cache.len(), 1);

        // Clear cache
        validator.clear_cache();
        assert_eq!(validator.auth_cache.len(), 0);
    }
}

/// Test security aspects
#[cfg(test)]
mod security_tests {
    use super::*;

    #[test]
    fn test_no_regex_injection() {
        // Test that regex patterns don't allow injection
        let malicious_comment = r#"🔴 Required Changes
**Issue Type**: [Bug]
**Severity**: [High]

### Description
Test with special regex characters: .*+?^${}()|[]\

### Acceptance Criteria Not Met
- [ ] Normal criterion"#;

        let parser = FeedbackParser::new();
        let result = parser.parse_comment(
            malicious_comment,
            "5DLabs-Tess",
            12345,
            678,
            "task-2",
        );

        // Should parse successfully without regex injection issues
        assert!(result.is_ok());
    }

    #[test]
    fn test_large_input_handling() {
        let parser = FeedbackParser::new();
        let large_comment = "x".repeat(1024 * 1024); // 1MB of 'x' characters

        let result = parser.parse_comment(
            &large_comment,
            "5DLabs-Tess",
            12345,
            678,
            "task-2",
        );

        // Should fail gracefully due to size limits, not crash
        assert!(result.is_err());
    }

    #[test]
    fn test_unauthorized_author_isolation() {
        let parser = FeedbackParser::new();

        // Ensure unauthorized authors can't access parsing
        let result = parser.parse_comment(
            COMPLETE_FEEDBACK_COMMENT,
            "malicious-user",
            12345,
            678,
            "task-2",
        );

        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), ParseError::UnauthorizedAuthor { .. }));
    }
}

/// Test concurrent access and thread safety
#[cfg(test)]
mod concurrency_tests {
    use super::*;
    use std::sync::Arc;
    use std::thread;

    #[test]
    fn test_concurrent_parsing() {
        let parser = Arc::new(FeedbackParser::new());
        let mut handles = vec![];

        for i in 0..10 {
            let parser_clone = Arc::clone(&parser);
            let handle = thread::spawn(move || {
                let result = parser_clone.parse_comment(
                    COMPLETE_FEEDBACK_COMMENT,
                    "5DLabs-Tess",
                    12345 + i,
                    678 + i,
                    &format!("task-{}", i),
                );
                assert!(result.is_ok());
            });
            handles.push(handle);
        }

        for handle in handles {
            handle.join().unwrap();
        }
    }

    #[test]
    fn test_shared_validator_thread_safety() {
        use crate::remediation::auth::SharedAuthorValidator;

        let shared_validator = SharedAuthorValidator::new();
        let mut handles = vec![];

        for i in 0..5 {
            let validator_clone = Arc::clone(&shared_validator);
            let handle = thread::spawn(move || {
                // Test validation
                let result = validator_clone.validate_author("5DLabs-Tess");
                assert!(result.is_ok());

                // Test adding author
                let author_name = format!("test-author-{}", i);
                let add_result = validator_clone.add_approved_author(&author_name);
                assert!(add_result.is_ok());

                // Test getting authors
                let authors = validator_clone.get_approved_authors().unwrap();
                assert!(authors.contains(&author_name));
            });
            handles.push(handle);
        }

        for handle in handles {
            handle.join().unwrap();
        }
    }
}
