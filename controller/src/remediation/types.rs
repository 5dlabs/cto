use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::cmp::Ordering;
use std::fmt;

/// Main structured feedback container representing parsed QA feedback
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StructuredFeedback {
    /// The type of issue identified (Bug, `MissingFeature`, etc.)
    pub issue_type: IssueType,
    /// The severity level of the issue
    pub severity: Severity,
    /// Detailed description of the issue
    pub description: String,
    /// List of acceptance criteria that are not met
    pub criteria_not_met: Vec<CriteriaStatus>,
    /// Optional reproduction steps
    pub reproduction_steps: Option<Vec<String>>,
    /// Optional expected behavior description
    pub expected_behavior: Option<String>,
    /// Optional actual behavior description
    pub actual_behavior: Option<String>,
    /// Metadata about the feedback source
    pub metadata: FeedbackMetadata,
}

/// Types of issues that can be identified in QA feedback
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum IssueType {
    /// Software bug or defect
    Bug,
    /// Missing feature or functionality
    MissingFeature,
    /// Regression from previous version
    Regression,
    /// Performance issue or degradation
    Performance,
}

impl fmt::Display for IssueType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            IssueType::Bug => write!(f, "Bug"),
            IssueType::MissingFeature => write!(f, "Missing Feature"),
            IssueType::Regression => write!(f, "Regression"),
            IssueType::Performance => write!(f, "Performance"),
        }
    }
}

/// Severity levels for issues, with ordering (Critical > High > Medium > Low)
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum Severity {
    /// Critical issue blocking functionality
    Critical,
    /// High priority issue affecting major features
    High,
    /// Medium priority issue affecting minor features
    Medium,
    /// Low priority issue or enhancement
    Low,
}

impl PartialOrd for Severity {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Severity {
    fn cmp(&self, other: &Self) -> Ordering {
        self.rank().cmp(&other.rank())
    }
}

impl Severity {
    const fn rank(&self) -> u8 {
        match self {
            Severity::Critical => 3,
            Severity::High => 2,
            Severity::Medium => 1,
            Severity::Low => 0,
        }
    }
}

impl fmt::Display for Severity {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Severity::Critical => write!(f, "Critical"),
            Severity::High => write!(f, "High"),
            Severity::Medium => write!(f, "Medium"),
            Severity::Low => write!(f, "Low"),
        }
    }
}

/// Status of an individual acceptance criterion
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CriteriaStatus {
    /// Description of the criterion
    pub description: String,
    /// Whether this criterion is completed
    pub completed: bool,
    /// Optional line number in the original comment
    pub line_number: Option<usize>,
}

impl CriteriaStatus {
    /// Create a new uncompleted criterion
    #[must_use]
    pub fn new(description: String) -> Self {
        Self {
            description,
            completed: false,
            line_number: None,
        }
    }

    /// Create a new criterion with specified completion status
    #[must_use]
    pub fn with_status(description: String, completed: bool) -> Self {
        Self {
            description,
            completed,
            line_number: None,
        }
    }

    /// Set the line number for this criterion
    #[must_use]
    pub fn with_line_number(mut self, line_number: usize) -> Self {
        self.line_number = Some(line_number);
        self
    }
}

/// Metadata about the feedback source and context
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FeedbackMetadata {
    /// Author of the feedback comment
    pub author: String,
    /// Timestamp when the feedback was created
    pub timestamp: DateTime<Utc>,
    /// GitHub comment ID
    pub comment_id: u64,
    /// Pull request number
    pub pr_number: u32,
    /// Task ID this feedback relates to
    pub task_id: String,
}

impl FeedbackMetadata {
    /// Create new metadata with current timestamp
    #[must_use]
    pub fn new(author: String, comment_id: u64, pr_number: u32, task_id: String) -> Self {
        Self {
            author,
            timestamp: Utc::now(),
            comment_id,
            pr_number,
            task_id,
        }
    }

    /// Create metadata with specific timestamp (useful for testing)
    #[must_use]
    pub fn with_timestamp(
        author: String,
        timestamp: DateTime<Utc>,
        comment_id: u64,
        pr_number: u32,
        task_id: String,
    ) -> Self {
        Self {
            author,
            timestamp,
            comment_id,
            pr_number,
            task_id,
        }
    }
}

/// Helper type for representing optional reproduction steps
pub type OptionalSteps = Option<Vec<String>>;

/// Helper type for representing optional behavior descriptions
pub type OptionalBehavior = Option<String>;

/// Type alias for parsing results
pub type ParseResult<T> = Result<T, ParseError>;

/// Placeholder for custom parse error - will be defined in error.rs
#[derive(Debug, thiserror::Error)]
pub enum ParseError {
    #[error("Placeholder error - will be replaced with full error types")]
    Placeholder,
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::TimeZone;

    #[test]
    fn test_issue_type_display() {
        assert_eq!(format!("{}", IssueType::Bug), "Bug");
        assert_eq!(format!("{}", IssueType::MissingFeature), "Missing Feature");
        assert_eq!(format!("{}", IssueType::Regression), "Regression");
        assert_eq!(format!("{}", IssueType::Performance), "Performance");
    }

    #[test]
    fn test_severity_display() {
        assert_eq!(format!("{}", Severity::Critical), "Critical");
        assert_eq!(format!("{}", Severity::High), "High");
        assert_eq!(format!("{}", Severity::Medium), "Medium");
        assert_eq!(format!("{}", Severity::Low), "Low");
    }

    #[test]
    fn test_severity_ordering() {
        assert!(Severity::Critical > Severity::High);
        assert!(Severity::High > Severity::Medium);
        assert!(Severity::Medium > Severity::Low);
        assert_eq!(Severity::Critical, Severity::Critical);
    }

    #[test]
    fn test_criteria_status_constructors() {
        let criterion = CriteriaStatus::new("Test criterion".to_string());
        assert_eq!(criterion.description, "Test criterion");
        assert!(!criterion.completed);
        assert!(criterion.line_number.is_none());

        let completed = CriteriaStatus::with_status("Completed criterion".to_string(), true);
        assert_eq!(completed.description, "Completed criterion");
        assert!(completed.completed);

        let with_line = criterion.with_line_number(42);
        assert_eq!(with_line.line_number, Some(42));
    }

    #[test]
    fn test_feedback_metadata_creation() {
        let metadata =
            FeedbackMetadata::new("test-author".to_string(), 12345, 678, "task-1".to_string());

        assert_eq!(metadata.author, "test-author");
        assert_eq!(metadata.comment_id, 12345);
        assert_eq!(metadata.pr_number, 678);
        assert_eq!(metadata.task_id, "task-1");
    }

    #[test]
    fn test_feedback_metadata_with_timestamp() {
        let timestamp = Utc.with_ymd_and_hms(2024, 1, 1, 12, 0, 0).unwrap();
        let metadata = FeedbackMetadata::with_timestamp(
            "test-author".to_string(),
            timestamp,
            12345,
            678,
            "task-1".to_string(),
        );

        assert_eq!(metadata.timestamp, timestamp);
    }

    #[test]
    fn test_structured_feedback_creation() {
        let metadata =
            FeedbackMetadata::new("5DLabs-Tess".to_string(), 12345, 678, "task-2".to_string());

        let criteria = vec![
            CriteriaStatus::new("User authentication works".to_string()),
            CriteriaStatus::with_status("Password reset functions".to_string(), true),
        ];

        let feedback = StructuredFeedback {
            issue_type: IssueType::Bug,
            severity: Severity::High,
            description: "Login button not working".to_string(),
            criteria_not_met: criteria,
            reproduction_steps: Some(vec![
                "Click login".to_string(),
                "Enter credentials".to_string(),
            ]),
            expected_behavior: Some("User should be logged in".to_string()),
            actual_behavior: Some("Page refreshes without login".to_string()),
            metadata,
        };

        assert_eq!(feedback.issue_type, IssueType::Bug);
        assert_eq!(feedback.severity, Severity::High);
        assert_eq!(feedback.criteria_not_met.len(), 2);
        assert!(feedback.reproduction_steps.is_some());
        assert!(feedback.expected_behavior.is_some());
        assert!(feedback.actual_behavior.is_some());
    }
}
