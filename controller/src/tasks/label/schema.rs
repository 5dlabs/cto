//! # Label Schema Definitions
//!
//! This module defines the comprehensive label schema for the Agent Remediation Loop,
//! including label types, patterns, lifecycle rules, and state machine definitions.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Represents the different types of labels used in the workflow
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum LabelType {
    /// Task association labels (task-{id})
    TaskAssociation,
    /// Iteration tracking labels (iteration-{n})
    IterationTracking,
    /// Workflow status labels
    Status,
    /// Human override labels
    Override,
}

/// Defines the schema for a specific label type
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LabelTypeSchema {
    /// The label type this schema defines
    pub label_type: LabelType,
    /// Pattern for matching labels of this type
    pub pattern: String,
    /// Example labels that match this pattern
    pub examples: Vec<String>,
    /// Lifecycle behavior for this label type
    pub lifecycle: LabelLifecycle,
    /// Purpose description
    pub purpose: String,
}

/// Defines how labels of a type behave over their lifecycle
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum LabelLifecycle {
    /// Label persists until explicitly removed
    Permanent,
    /// Label is updated per remediation cycle
    UpdatedPerCycle,
    /// Label is managed based on workflow state
    StateBased,
    /// Label is managed by human operators
    ManualControl,
}

/// Workflow states in the remediation process
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum WorkflowState {
    /// Initial state - no remediation loop running yet
    Initial,
    /// Tess (or a human) has requested fixes from Rex
    NeedsFixes,
    /// Rex is actively working on remediation fixes
    FixingInProgress,
    /// Rex completed the pass and Cleo needs to rerun quality checks
    NeedsCleo,
    /// Cleo approved and Tess needs to re-review
    NeedsTess,
    /// Tess has approved the changes
    Approved,
    /// Remediation failed (manual stop or iteration cap)
    Failed,
    /// Human override is active
    ManualOverride,
}

/// Defines a transition between workflow states
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StateTransition {
    /// State to transition from
    pub from: WorkflowState,
    /// State to transition to
    pub to: WorkflowState,
    /// Event that triggers this transition
    pub trigger: String,
    /// Conditions that must be met for transition
    pub conditions: Vec<String>,
    /// Actions to perform during transition
    pub actions: Vec<String>,
}

/// Operations that can be performed on labels
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum LabelOperationType {
    /// Add one or more labels
    Add,
    /// Remove one or more labels
    Remove,
    /// Replace one label with another
    Replace,
}

/// Represents a single label operation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LabelOperation {
    /// Type of operation to perform
    pub operation_type: LabelOperationType,
    /// Labels to operate on
    pub labels: Vec<String>,
    /// Source label for replace operations
    pub from_label: Option<String>,
}

/// Complete label schema for the workflow system
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LabelSchema {
    /// Schema definitions for each label type
    pub type_schemas: HashMap<LabelType, LabelTypeSchema>,
    /// All valid workflow states
    pub workflow_states: Vec<WorkflowState>,
    /// All valid state transitions
    pub state_transitions: Vec<StateTransition>,
    /// Status labels and their meanings
    pub status_labels: HashMap<String, String>,
    /// Override labels and their behaviors
    pub override_labels: HashMap<String, OverrideBehavior>,
}

/// Defines behavior for override labels
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OverrideBehavior {
    /// Description of the override
    pub description: String,
    /// Severity level (low, medium, high)
    pub severity: String,
    /// Action to take when override is detected
    pub action: String,
}

impl Default for LabelSchema {
    fn default() -> Self {
        let mut type_schemas = HashMap::new();

        // Task association labels
        type_schemas.insert(
            LabelType::TaskAssociation,
            LabelTypeSchema {
                label_type: LabelType::TaskAssociation,
                pattern: "task-\\d+".to_string(),
                examples: vec!["task-42".to_string(), "task-123".to_string()],
                lifecycle: LabelLifecycle::Permanent,
                purpose: "task association".to_string(),
            },
        );

        // Iteration tracking labels
        type_schemas.insert(
            LabelType::IterationTracking,
            LabelTypeSchema {
                label_type: LabelType::IterationTracking,
                pattern: "iteration-\\d+".to_string(),
                examples: vec!["iteration-1".to_string(), "iteration-3".to_string()],
                lifecycle: LabelLifecycle::UpdatedPerCycle,
                purpose: "iteration tracking".to_string(),
            },
        );

        // Status labels
        type_schemas.insert(
            LabelType::Status,
            LabelTypeSchema {
                label_type: LabelType::Status,
                pattern:
                    "(needs-fixes|fixing-in-progress|needs-cleo|needs-tess|approved|failed-remediation)"
                        .to_string(),
                examples: vec![
                    "needs-fixes".to_string(),
                    "fixing-in-progress".to_string(),
                    "needs-cleo".to_string(),
                    "needs-tess".to_string(),
                ],
                lifecycle: LabelLifecycle::StateBased,
                purpose: "workflow status".to_string(),
            },
        );

        // Override labels
        type_schemas.insert(
            LabelType::Override,
            LabelTypeSchema {
                label_type: LabelType::Override,
                pattern: "(skip-automation|manual-review-required|pause-remediation)".to_string(),
                examples: vec![
                    "skip-automation".to_string(),
                    "manual-review-required".to_string(),
                ],
                lifecycle: LabelLifecycle::ManualControl,
                purpose: "human override".to_string(),
            },
        );

        let workflow_states = vec![
            WorkflowState::Initial,
            WorkflowState::NeedsFixes,
            WorkflowState::FixingInProgress,
            WorkflowState::NeedsCleo,
            WorkflowState::NeedsTess,
            WorkflowState::Approved,
            WorkflowState::Failed,
            WorkflowState::ManualOverride,
        ];

        let state_transitions = vec![
            StateTransition {
                from: WorkflowState::Initial,
                to: WorkflowState::NeedsFixes,
                trigger: "tess_changes_requested".to_string(),
                conditions: vec![],
                actions: vec![
                    "add_needs_fixes".to_string(),
                    "increment_iteration".to_string(),
                ],
            },
            StateTransition {
                from: WorkflowState::NeedsFixes,
                to: WorkflowState::FixingInProgress,
                trigger: "rex_work_started".to_string(),
                conditions: vec![],
                actions: vec![
                    "remove_needs_fixes".to_string(),
                    "add_fixing_in_progress".to_string(),
                ],
            },
            StateTransition {
                from: WorkflowState::FixingInProgress,
                to: WorkflowState::NeedsCleo,
                trigger: "rex_work_completed".to_string(),
                conditions: vec![],
                actions: vec![
                    "remove_fixing_in_progress".to_string(),
                    "add_needs_cleo".to_string(),
                ],
            },
            StateTransition {
                from: WorkflowState::NeedsCleo,
                to: WorkflowState::NeedsTess,
                trigger: "cleo_checks_passed".to_string(),
                conditions: vec![],
                actions: vec![
                    "remove_needs_cleo".to_string(),
                    "add_needs_tess".to_string(),
                ],
            },
            StateTransition {
                from: WorkflowState::NeedsCleo,
                to: WorkflowState::NeedsFixes,
                trigger: "cleo_changes_requested".to_string(),
                conditions: vec![],
                actions: vec![
                    "remove_needs_cleo".to_string(),
                    "add_needs_fixes".to_string(),
                    "increment_iteration".to_string(),
                ],
            },
            StateTransition {
                from: WorkflowState::NeedsTess,
                to: WorkflowState::NeedsFixes,
                trigger: "tess_changes_requested".to_string(),
                conditions: vec![],
                actions: vec![
                    "remove_needs_tess".to_string(),
                    "add_needs_fixes".to_string(),
                    "increment_iteration".to_string(),
                ],
            },
            StateTransition {
                from: WorkflowState::NeedsTess,
                to: WorkflowState::Approved,
                trigger: "tess_approved".to_string(),
                conditions: vec![],
                actions: vec!["remove_needs_tess".to_string(), "add_approved".to_string()],
            },
            StateTransition {
                from: WorkflowState::FixingInProgress,
                to: WorkflowState::Failed,
                trigger: "max_iterations_reached".to_string(),
                conditions: vec!["iteration >= 10".to_string()],
                actions: vec![
                    "remove_fixing_in_progress".to_string(),
                    "add_failed_remediation".to_string(),
                ],
            },
            StateTransition {
                from: WorkflowState::NeedsFixes,
                to: WorkflowState::Failed,
                trigger: "max_iterations_reached".to_string(),
                conditions: vec!["iteration >= 10".to_string()],
                actions: vec![
                    "remove_needs_fixes".to_string(),
                    "add_failed_remediation".to_string(),
                ],
            },
            StateTransition {
                from: WorkflowState::NeedsTess,
                to: WorkflowState::Failed,
                trigger: "max_iterations_reached".to_string(),
                conditions: vec!["iteration >= 10".to_string()],
                actions: vec![
                    "remove_needs_tess".to_string(),
                    "add_failed_remediation".to_string(),
                ],
            },
            StateTransition {
                from: WorkflowState::NeedsCleo,
                to: WorkflowState::Failed,
                trigger: "max_iterations_reached".to_string(),
                conditions: vec!["iteration >= 10".to_string()],
                actions: vec![
                    "remove_needs_cleo".to_string(),
                    "add_failed_remediation".to_string(),
                ],
            },
        ];

        let mut status_labels = HashMap::new();
        status_labels.insert(
            "needs-fixes".to_string(),
            "Tess (or human) requested Rex to remediate findings".to_string(),
        );
        status_labels.insert(
            "fixing-in-progress".to_string(),
            "Rex is currently remediating the outstanding findings".to_string(),
        );
        status_labels.insert(
            "needs-cleo".to_string(),
            "Rex finished the pass; Cleo must re-run quality checks".to_string(),
        );
        status_labels.insert(
            "needs-tess".to_string(),
            "Cleo approved; Tess needs to re-review the changes".to_string(),
        );
        status_labels.insert(
            "approved".to_string(),
            "Tess approved the changes".to_string(),
        );
        status_labels.insert(
            "failed-remediation".to_string(),
            "Max iterations reached, remediation failed".to_string(),
        );

        let mut override_labels = HashMap::new();
        override_labels.insert(
            "skip-automation".to_string(),
            OverrideBehavior {
                description: "Disables all automated workflows".to_string(),
                severity: "high".to_string(),
                action: "halt_all_automation".to_string(),
            },
        );
        override_labels.insert(
            "manual-review-required".to_string(),
            OverrideBehavior {
                description: "Manual review required before automation continues".to_string(),
                severity: "medium".to_string(),
                action: "pause_until_review".to_string(),
            },
        );
        override_labels.insert(
            "pause-remediation".to_string(),
            OverrideBehavior {
                description: "Remediation temporarily paused".to_string(),
                severity: "low".to_string(),
                action: "pause_remediation_only".to_string(),
            },
        );

        Self {
            type_schemas,
            workflow_states,
            state_transitions,
            status_labels,
            override_labels,
        }
    }
}

impl LabelSchema {
    /// Validate that a label matches the expected pattern for its type
    #[must_use] pub fn validate_label(&self, label: &str, label_type: &LabelType) -> bool {
        if let Some(schema) = self.type_schemas.get(label_type) {
            let regex_pattern = format!("^{}$", schema.pattern);
            regex::Regex::new(&regex_pattern)
                .map(|re| re.is_match(label))
                .unwrap_or(false)
        } else {
            false
        }
    }

    /// Determine the type of a label based on its pattern
    #[must_use] pub fn classify_label(&self, label: &str) -> Option<LabelType> {
        for label_type in self.type_schemas.keys() {
            if self.validate_label(label, label_type) {
                return Some(label_type.clone());
            }
        }

        // Check for exact status label matches
        if self.status_labels.contains_key(label) {
            return Some(LabelType::Status);
        }

        // Check for exact override label matches
        if self.override_labels.contains_key(label) {
            return Some(LabelType::Override);
        }

        None
    }

    /// Get the current workflow state from a set of labels
    #[must_use] pub fn determine_workflow_state(&self, labels: &[String]) -> WorkflowState {
        if labels.contains(&"approved".to_string()) {
            WorkflowState::Approved
        } else if labels.contains(&"failed-remediation".to_string()) {
            WorkflowState::Failed
        } else if labels.contains(&"needs-tess".to_string()) {
            WorkflowState::NeedsTess
        } else if labels.contains(&"needs-cleo".to_string()) {
            WorkflowState::NeedsCleo
        } else if labels.contains(&"fixing-in-progress".to_string()) {
            WorkflowState::FixingInProgress
        } else if labels.contains(&"needs-fixes".to_string()) {
            WorkflowState::NeedsFixes
        } else {
            WorkflowState::Initial
        }
    }

    /// Check if a state transition is valid
    #[must_use] pub fn is_valid_transition(
        &self,
        from: &WorkflowState,
        to: &WorkflowState,
        trigger: &str,
    ) -> bool {
        self.state_transitions.iter().any(|transition| {
            &transition.from == from && &transition.to == to && transition.trigger == trigger
        })
    }

    /// Get the transition definition for a specific transition
    #[must_use] pub fn get_transition(
        &self,
        from: &WorkflowState,
        to: &WorkflowState,
        trigger: &str,
    ) -> Option<&StateTransition> {
        self.state_transitions.iter().find(|transition| {
            &transition.from == from && &transition.to == to && transition.trigger == trigger
        })
    }

    /// Check if a workflow state is terminal (end state)
    #[must_use] pub fn is_terminal_state(&self, state: &WorkflowState) -> bool {
        matches!(state, WorkflowState::Approved | WorkflowState::Failed)
    }
}
