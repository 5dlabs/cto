//! Decision point and record entities for captured discovery.
//!
//! This module implements the "Captured Discovery" pattern, which surfaces
//! decision points during PRD intake and requires explicit decision logging
//! during implementation.
//!
//! Key concepts:
//! - **DecisionPoint**: A predicted decision that will need to be made during implementation
//! - **DecisionRecord**: A record of a decision actually made, with rationale
//! - **DecisionCategory**: The type of decision (architecture, error handling, etc.)
//! - **ConstraintType**: How constrained the decision is (hard, soft, open, escalation)

use serde::{Deserialize, Serialize};

use crate::errors::TasksError;

/// Category of decision point.
///
/// Used to classify decisions for filtering, reporting, and routing
/// to appropriate reviewers.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
#[serde(rename_all = "kebab-case")]
pub enum DecisionCategory {
    /// System design, patterns, structure choices
    #[default]
    Architecture,
    /// Failure modes, recovery strategies, error responses
    ErrorHandling,
    /// Schema design, types, relationships, migrations
    DataModel,
    /// Endpoints, contracts, versioning, backwards compatibility
    ApiDesign,
    /// Empty states, loading states, user interactions, feedback
    UxBehavior,
    /// Caching, batching, optimization, resource usage
    Performance,
    /// Auth, validation, encryption, access control
    Security,
}

impl std::fmt::Display for DecisionCategory {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Architecture => write!(f, "architecture"),
            Self::ErrorHandling => write!(f, "error-handling"),
            Self::DataModel => write!(f, "data-model"),
            Self::ApiDesign => write!(f, "api-design"),
            Self::UxBehavior => write!(f, "ux-behavior"),
            Self::Performance => write!(f, "performance"),
            Self::Security => write!(f, "security"),
        }
    }
}

impl std::str::FromStr for DecisionCategory {
    type Err = TasksError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().replace('_', "-").as_str() {
            "architecture" | "arch" => Ok(Self::Architecture),
            "error-handling" | "error" | "errors" => Ok(Self::ErrorHandling),
            "data-model" | "data" | "schema" => Ok(Self::DataModel),
            "api-design" | "api" => Ok(Self::ApiDesign),
            "ux-behavior" | "ux" | "ui" => Ok(Self::UxBehavior),
            "performance" | "perf" => Ok(Self::Performance),
            "security" | "sec" | "auth" => Ok(Self::Security),
            _ => Err(TasksError::ValidationError {
                field: "category".to_string(),
                reason: format!("unknown decision category: {s}"),
            }),
        }
    }
}

/// Constraint type for guidance classification.
///
/// Determines how much freedom the agent has in making a decision.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
#[serde(rename_all = "kebab-case")]
pub enum ConstraintType {
    /// Must be exactly this way - no deviation allowed
    Hard,
    /// Prefer this approach, but agent can adjust with justification
    #[default]
    Soft,
    /// Agent chooses freely, but must log rationale
    Open,
    /// Human must decide before implementation proceeds
    Escalation,
}

impl std::fmt::Display for ConstraintType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Hard => write!(f, "hard"),
            Self::Soft => write!(f, "soft"),
            Self::Open => write!(f, "open"),
            Self::Escalation => write!(f, "escalation"),
        }
    }
}

impl std::str::FromStr for ConstraintType {
    type Err = TasksError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "hard" | "required" | "must" => Ok(Self::Hard),
            "soft" | "preferred" | "should" => Ok(Self::Soft),
            "open" | "free" | "any" => Ok(Self::Open),
            "escalation" | "escalate" | "human" | "approval" => Ok(Self::Escalation),
            _ => Err(TasksError::ValidationError {
                field: "constraint_type".to_string(),
                reason: format!("unknown constraint type: {s}"),
            }),
        }
    }
}

/// A predicted decision point where agent judgment is required.
///
/// Decision points are identified during PRD intake and surface areas
/// where the implementation agent will need to make choices. This enables
/// "captured discovery" - surfacing decisions before they're made rather
/// than discovering them buried in code later.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DecisionPoint {
    /// Unique identifier within the task (e.g., "d1", "d2")
    pub id: String,

    /// Category of decision
    pub category: DecisionCategory,

    /// Description of what needs to be decided
    pub description: String,

    /// Known options to consider (may be empty if truly open-ended)
    #[serde(default)]
    pub options: Vec<String>,

    /// Whether human approval is required before proceeding
    #[serde(default, rename = "requiresApproval")]
    pub requires_approval: bool,

    /// Constraints or guidance from the PRD
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub constraints: Option<String>,

    /// How constrained this decision is
    #[serde(default, rename = "constraintType")]
    pub constraint_type: ConstraintType,
}

impl DecisionPoint {
    /// Create a new decision point with minimal required fields.
    #[must_use]
    pub fn new(
        id: impl Into<String>,
        category: DecisionCategory,
        description: impl Into<String>,
    ) -> Self {
        Self {
            id: id.into(),
            category,
            description: description.into(),
            options: Vec::new(),
            requires_approval: false,
            constraints: None,
            constraint_type: ConstraintType::default(),
        }
    }

    /// Add options to consider.
    #[must_use]
    pub fn with_options(mut self, options: Vec<String>) -> Self {
        self.options = options;
        self
    }

    /// Mark as requiring human approval.
    #[must_use]
    pub fn with_approval_required(mut self) -> Self {
        self.requires_approval = true;
        self.constraint_type = ConstraintType::Escalation;
        self
    }

    /// Add constraints from PRD.
    #[must_use]
    pub fn with_constraints(mut self, constraints: impl Into<String>) -> Self {
        self.constraints = Some(constraints.into());
        self
    }

    /// Set constraint type.
    #[must_use]
    pub fn with_constraint_type(mut self, constraint_type: ConstraintType) -> Self {
        self.constraint_type = constraint_type;
        self
    }

    /// Check if this is an escalation decision requiring human input.
    #[must_use]
    pub fn is_escalation(&self) -> bool {
        self.requires_approval || self.constraint_type == ConstraintType::Escalation
    }
}

/// A recorded decision made during implementation.
///
/// Decision records capture what was decided, why, and what alternatives
/// were considered. This creates an audit trail and enables review of
/// agent judgment calls.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DecisionRecord {
    /// The decision that was made
    pub decision: String,

    /// Category of decision
    pub category: DecisionCategory,

    /// Alternatives that were considered
    #[serde(default, rename = "alternativesConsidered")]
    pub alternatives_considered: Vec<String>,

    /// Why this choice was made
    pub rationale: String,

    /// Confidence level (1-5, where 5 is highest confidence)
    pub confidence: u8,

    /// Whether this was reviewed by a human
    #[serde(default, rename = "humanReviewed")]
    pub human_reviewed: bool,

    /// Related decision point ID if this was a predicted decision
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        rename = "decisionPointId"
    )]
    pub decision_point_id: Option<String>,

    /// When the decision was made (ISO 8601 timestamp)
    #[serde(rename = "madeAt")]
    pub made_at: String,
}

impl DecisionRecord {
    /// Create a new decision record.
    #[must_use]
    pub fn new(
        decision: impl Into<String>,
        category: DecisionCategory,
        rationale: impl Into<String>,
    ) -> Self {
        Self {
            decision: decision.into(),
            category,
            alternatives_considered: Vec::new(),
            rationale: rationale.into(),
            confidence: 3, // Default to medium confidence
            human_reviewed: false,
            decision_point_id: None,
            made_at: chrono::Utc::now().to_rfc3339(),
        }
    }

    /// Add alternatives that were considered.
    #[must_use]
    pub fn with_alternatives(mut self, alternatives: Vec<String>) -> Self {
        self.alternatives_considered = alternatives;
        self
    }

    /// Set confidence level (clamped to 1-5).
    #[must_use]
    pub fn with_confidence(mut self, confidence: u8) -> Self {
        self.confidence = confidence.clamp(1, 5);
        self
    }

    /// Link to a predicted decision point.
    #[must_use]
    pub fn with_decision_point(mut self, decision_point_id: impl Into<String>) -> Self {
        self.decision_point_id = Some(decision_point_id.into());
        self
    }

    /// Mark as human reviewed.
    #[must_use]
    pub fn with_human_review(mut self) -> Self {
        self.human_reviewed = true;
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_decision_category_display() {
        assert_eq!(DecisionCategory::Architecture.to_string(), "architecture");
        assert_eq!(
            DecisionCategory::ErrorHandling.to_string(),
            "error-handling"
        );
        assert_eq!(DecisionCategory::UxBehavior.to_string(), "ux-behavior");
    }

    #[test]
    fn test_decision_category_from_str() {
        assert_eq!(
            "architecture".parse::<DecisionCategory>().unwrap(),
            DecisionCategory::Architecture
        );
        assert_eq!(
            "error-handling".parse::<DecisionCategory>().unwrap(),
            DecisionCategory::ErrorHandling
        );
        assert_eq!(
            "ux".parse::<DecisionCategory>().unwrap(),
            DecisionCategory::UxBehavior
        );
        assert!("invalid".parse::<DecisionCategory>().is_err());
    }

    #[test]
    fn test_constraint_type_display() {
        assert_eq!(ConstraintType::Hard.to_string(), "hard");
        assert_eq!(ConstraintType::Escalation.to_string(), "escalation");
    }

    #[test]
    fn test_constraint_type_from_str() {
        assert_eq!(
            "hard".parse::<ConstraintType>().unwrap(),
            ConstraintType::Hard
        );
        assert_eq!(
            "escalation".parse::<ConstraintType>().unwrap(),
            ConstraintType::Escalation
        );
        assert_eq!(
            "human".parse::<ConstraintType>().unwrap(),
            ConstraintType::Escalation
        );
        assert!("invalid".parse::<ConstraintType>().is_err());
    }

    #[test]
    fn test_decision_point_new() {
        let dp = DecisionPoint::new(
            "d1",
            DecisionCategory::Architecture,
            "How to structure the API",
        );
        assert_eq!(dp.id, "d1");
        assert_eq!(dp.category, DecisionCategory::Architecture);
        assert!(!dp.requires_approval);
        assert_eq!(dp.constraint_type, ConstraintType::Soft);
    }

    #[test]
    fn test_decision_point_builder() {
        let dp = DecisionPoint::new("d1", DecisionCategory::UxBehavior, "Empty state design")
            .with_options(vec!["Option A".to_string(), "Option B".to_string()])
            .with_approval_required()
            .with_constraints("Must be accessible");

        assert_eq!(dp.options.len(), 2);
        assert!(dp.requires_approval);
        assert_eq!(dp.constraint_type, ConstraintType::Escalation);
        assert_eq!(dp.constraints, Some("Must be accessible".to_string()));
    }

    #[test]
    fn test_decision_point_is_escalation() {
        let dp1 = DecisionPoint::new("d1", DecisionCategory::Architecture, "Test")
            .with_approval_required();
        assert!(dp1.is_escalation());

        let dp2 = DecisionPoint::new("d2", DecisionCategory::Architecture, "Test")
            .with_constraint_type(ConstraintType::Escalation);
        assert!(dp2.is_escalation());

        let dp3 = DecisionPoint::new("d3", DecisionCategory::Architecture, "Test");
        assert!(!dp3.is_escalation());
    }

    #[test]
    fn test_decision_record_new() {
        let dr = DecisionRecord::new(
            "Use Redis for caching",
            DecisionCategory::Architecture,
            "Team already has Redis infrastructure",
        );
        assert_eq!(dr.decision, "Use Redis for caching");
        assert_eq!(dr.confidence, 3);
        assert!(!dr.human_reviewed);
    }

    #[test]
    fn test_decision_record_builder() {
        let dr = DecisionRecord::new(
            "Use retry with backoff",
            DecisionCategory::ErrorHandling,
            "Handles transient failures",
        )
        .with_alternatives(vec![
            "Fail immediately".to_string(),
            "Queue for later".to_string(),
        ])
        .with_confidence(4)
        .with_decision_point("d1")
        .with_human_review();

        assert_eq!(dr.alternatives_considered.len(), 2);
        assert_eq!(dr.confidence, 4);
        assert_eq!(dr.decision_point_id, Some("d1".to_string()));
        assert!(dr.human_reviewed);
    }

    #[test]
    fn test_decision_record_confidence_clamped() {
        let dr1 =
            DecisionRecord::new("Test", DecisionCategory::Architecture, "Test").with_confidence(10);
        assert_eq!(dr1.confidence, 5);

        let dr2 =
            DecisionRecord::new("Test", DecisionCategory::Architecture, "Test").with_confidence(0);
        assert_eq!(dr2.confidence, 1);
    }

    #[test]
    fn test_decision_point_serde() {
        let dp = DecisionPoint::new(
            "d1",
            DecisionCategory::ErrorHandling,
            "How to handle errors",
        )
        .with_options(vec!["Retry".to_string(), "Fail".to_string()])
        .with_constraint_type(ConstraintType::Open);

        let json = serde_json::to_string(&dp).unwrap();
        assert!(json.contains("\"id\":\"d1\""));
        assert!(json.contains("\"category\":\"error-handling\""));
        assert!(json.contains("\"constraintType\":\"open\""));

        let parsed: DecisionPoint = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.id, "d1");
        assert_eq!(parsed.category, DecisionCategory::ErrorHandling);
    }

    #[test]
    fn test_decision_record_serde() {
        let dr = DecisionRecord::new(
            "Use PostgreSQL",
            DecisionCategory::DataModel,
            "ACID compliance required",
        )
        .with_decision_point("d2");

        let json = serde_json::to_string(&dr).unwrap();
        assert!(json.contains("\"decision\":\"Use PostgreSQL\""));
        assert!(json.contains("\"category\":\"data-model\""));
        assert!(json.contains("\"decisionPointId\":\"d2\""));

        let parsed: DecisionRecord = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.decision, "Use PostgreSQL");
        assert_eq!(parsed.decision_point_id, Some("d2".to_string()));
    }
}
