//! Test fixtures and sample data for remediation parser tests

use chrono::{TimeZone, Utc};
use crate::remediation::types::{StructuredFeedback, IssueType, Severity, CriteriaStatus, FeedbackMetadata};

/// Complete sample feedback comment for testing
pub const COMPLETE_FEEDBACK_COMMENT: &str = r#"🔴 Required Changes
**Issue Type**: [Bug]
**Severity**: [High]

### Description
The login button is not working properly when users click it. This is a critical user experience issue that affects the primary authentication flow.

### Acceptance Criteria Not Met
- [ ] User authentication works properly after clicking login button
- [ ] Error messages are displayed for invalid credentials
- [x] Login form renders correctly on all screen sizes
- [ ] Password reset functionality works end-to-end

### Steps to Reproduce
1. Navigate to the login page at /login
2. Enter valid email address in the email field
3. Enter valid password in the password field
4. Click the "Login" button
5. Observe that the page refreshes without any login attempt

### Expected vs Actual
- **Expected**: User should be authenticated and redirected to the dashboard page
- **Actual**: Page refreshes with no authentication attempt, user remains on login page"#;

/// Minimal feedback comment for basic parsing tests
pub const MINIMAL_FEEDBACK_COMMENT: &str = r#"🔴 Required Changes
**Issue Type**: [Bug]
**Severity**: [Low]

### Description
Simple test case.

### Acceptance Criteria Not Met
- [ ] Test criterion"#;

/// Feedback comment missing required fields for error testing
pub const INCOMPLETE_FEEDBACK_COMMENT: &str = r#"🔴 Required Changes
**Issue Type**: [Bug]

### Description
Missing severity field.

### Acceptance Criteria Not Met
- [ ] Test criterion"#;

/// Feedback comment without actionable marker
pub const NON_ACTIONABLE_COMMENT: &str = r#"This is just a regular comment without the required changes marker.

**Issue Type**: [Bug]
**Severity**: [High]

### Description
This should not be parsed as it's missing the marker."#;

/// Feedback comment with malformed markdown
pub const MALFORMED_COMMENT: &str = r#"🔴 Required Changes
**Issue Type**: [Bug]
**Severity**: [High

### Description
Unclosed bracket in severity field.

### Acceptance Criteria Not Met
- [ ] Test criterion"#;

/// Large feedback comment for performance testing
pub fn create_large_comment(size_kb: usize) -> String {
    let base_comment = r#"🔴 Required Changes
**Issue Type**: [Performance]
**Severity**: [Medium]

### Description
This is a performance test with a large description."#;

    let mut large_description = String::new();
    while large_description.len() < size_kb * 1024 {
        large_description.push_str("This is additional content to make the comment larger for performance testing. ");
    }

    format!("{}\n\n{}", base_comment, large_description)
}

/// Create expected structured feedback for COMPLETE_FEEDBACK_COMMENT
pub fn expected_complete_feedback() -> StructuredFeedback {
    let metadata = FeedbackMetadata {
        author: "5DLabs-Tess".to_string(),
        timestamp: Utc::now(),
        comment_id: 12345,
        pr_number: 678,
        task_id: "task-2".to_string(),
    };

    let criteria = vec![
        CriteriaStatus {
            description: "User authentication works properly after clicking login button".to_string(),
            completed: false,
            line_number: Some(1),
        },
        CriteriaStatus {
            description: "Error messages are displayed for invalid credentials".to_string(),
            completed: false,
            line_number: Some(2),
        },
        CriteriaStatus {
            description: "Login form renders correctly on all screen sizes".to_string(),
            completed: true,
            line_number: Some(3),
        },
        CriteriaStatus {
            description: "Password reset functionality works end-to-end".to_string(),
            completed: false,
            line_number: Some(4),
        },
    ];

    let reproduction_steps = Some(vec![
        "Navigate to the login page at /login".to_string(),
        "Enter valid email address in the email field".to_string(),
        "Enter valid password in the password field".to_string(),
        "Click the \"Login\" button".to_string(),
        "Observe that the page refreshes without any login attempt".to_string(),
    ]);

    StructuredFeedback {
        issue_type: IssueType::Bug,
        severity: Severity::High,
        description: "The login button is not working properly when users click it. This is a critical user experience issue that affects the primary authentication flow.".to_string(),
        criteria_not_met: criteria,
        reproduction_steps,
        expected_behavior: Some("User should be authenticated and redirected to the dashboard page".to_string()),
        actual_behavior: Some("Page refreshes with no authentication attempt, user remains on login page".to_string()),
        metadata,
    }
}

/// Create expected structured feedback for MINIMAL_FEEDBACK_COMMENT
pub fn expected_minimal_feedback() -> StructuredFeedback {
    let metadata = FeedbackMetadata {
        author: "5DLabs-Tess".to_string(),
        timestamp: Utc::now(),
        comment_id: 54321,
        pr_number: 999,
        task_id: "task-test".to_string(),
    };

    let criteria = vec![
        CriteriaStatus {
            description: "Test criterion".to_string(),
            completed: false,
            line_number: Some(1),
        },
    ];

    StructuredFeedback {
        issue_type: IssueType::Bug,
        severity: Severity::Low,
        description: "Simple test case.".to_string(),
        criteria_not_met: criteria,
        reproduction_steps: None,
        expected_behavior: None,
        actual_behavior: None,
        metadata,
    }
}

/// Test data for different issue types
pub const BUG_FEEDBACK: &str = r#"🔴 Required Changes
**Issue Type**: [Bug]
**Severity**: [High]

### Description
Bug report.

### Acceptance Criteria Not Met
- [ ] Fix the bug"#;

pub const FEATURE_FEEDBACK: &str = r#"🔴 Required Changes
**Issue Type**: [Missing Feature]
**Severity**: [Medium]

### Description
Missing feature request.

### Acceptance Criteria Not Met
- [ ] Implement the feature"#;

pub const REGRESSION_FEEDBACK: &str = r#"🔴 Required Changes
**Issue Type**: [Regression]
**Severity**: [Critical]

### Description
Regression issue.

### Acceptance Criteria Not Met
- [ ] Fix the regression"#;

pub const PERFORMANCE_FEEDBACK: &str = r#"🔴 Required Changes
**Issue Type**: [Performance]
**Severity**: [High]

### Description
Performance issue.

### Acceptance Criteria Not Met
- [ ] Optimize performance"#;

/// Test data for different severity levels
pub const CRITICAL_SEVERITY: &str = r#"🔴 Required Changes
**Issue Type**: [Bug]
**Severity**: [Critical]

### Description
Critical issue.

### Acceptance Criteria Not Met
- [ ] Fix critical issue"#;

pub const HIGH_SEVERITY: &str = r#"🔴 Required Changes
**Issue Type**: [Bug]
**Severity**: [High]

### Description
High priority issue.

### Acceptance Criteria Not Met
- [ ] Fix high priority issue"#;

pub const MEDIUM_SEVERITY: &str = r#"🔴 Required Changes
**Issue Type**: [Bug]
**Severity**: [Medium]

### Description
Medium priority issue.

### Acceptance Criteria Not Met
- [ ] Fix medium priority issue"#;

pub const LOW_SEVERITY: &str = r#"🔴 Required Changes
**Issue Type**: [Bug]
**Severity**: [Low]

### Description
Low priority issue.

### Acceptance Criteria Not Met
- [ ] Fix low priority issue"#;

/// Comments with different author scenarios
pub const AUTHORIZED_AUTHOR_COMMENT: &str = r#"🔴 Required Changes
**Issue Type**: [Bug]
**Severity**: [High]

### Description
Authorized author test.

### Acceptance Criteria Not Met
- [ ] Test criterion"#;

pub const UNAUTHORIZED_AUTHOR_COMMENT: &str = AUTHORIZED_AUTHOR_COMMENT;

/// Malformed comments for error testing
pub const MISSING_ISSUE_TYPE: &str = r#"🔴 Required Changes
**Severity**: [High]

### Description
Missing issue type.

### Acceptance Criteria Not Met
- [ ] Test criterion"#;

pub const MISSING_SEVERITY: &str = r#"🔴 Required Changes
**Issue Type**: [Bug]

### Description
Missing severity.

### Acceptance Criteria Not Met
- [ ] Test criterion"#;

pub const MISSING_DESCRIPTION: &str = r#"🔴 Required Changes
**Issue Type**: [Bug]
**Severity**: [High]

### Acceptance Criteria Not Met
- [ ] Test criterion"#;

pub const MISSING_CRITERIA: &str = r#"🔴 Required Changes
**Issue Type**: [Bug]
**Severity**: [High]

### Description
Missing criteria section."#;

/// Comments with different markdown formats
pub const ALTERNATIVE_CHECKBOX_FORMAT: &str = r#"🔴 Required Changes
**Issue Type**: [Bug]
**Severity**: [High]

### Description
Alternative checkbox format.

### Acceptance Criteria Not Met
* [ ] Asterisk format
1. [x] Numbered format
- [ ] Standard format"#;

/// Comments with special characters and edge cases
pub const UNICODE_COMMENT: &str = r#"🔴 Required Changes
**Issue Type**: [Bug]
**Severity**: [High]

### Description
Unicode test: 🚀 éñ∑∂🔥

### Acceptance Criteria Not Met
- [ ] Unicode criterion: αβγδε"#;

pub const EMPTY_SECTIONS: &str = r#"🔴 Required Changes
**Issue Type**: [Bug]
**Severity**: [High]

### Description
Test with empty sections.

### Acceptance Criteria Not Met

### Steps to Reproduce

### Expected vs Actual"#;

pub const WHITESPACE_COMMENT: &str = r#"🔴 Required Changes
**Issue Type**: [Bug]
**Severity**: [High]

### Description
    Test with    extra    whitespace    and    indentation.

### Acceptance Criteria Not Met
    - [ ]     Extra whitespace criterion     "#;
