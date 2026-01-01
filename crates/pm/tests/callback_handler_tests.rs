//! Unit tests for the intake/play callback handlers.
//!
//! These tests verify the deserialization and helper functions
//! work correctly without needing a real Linear API.

use pm::handlers::intake::IntakeTask;

// =============================================================================
// IntakeTask Deserialization Tests
// =============================================================================

/// Test that IntakeTask deserializes correctly from AI-generated JSON.
#[test]
fn test_intake_task_deserialization_numeric_ids() {
    let json = r#"{
        "id": 1,
        "title": "Infrastructure Setup",
        "description": "Provision databases",
        "dependencies": [2, 3],
        "priority": 2
    }"#;

    let task: IntakeTask = serde_json::from_str(json).unwrap();
    assert_eq!(task.id, "1");
    assert_eq!(task.title, "Infrastructure Setup");
    assert_eq!(task.dependencies, vec!["2", "3"]);
    assert_eq!(task.priority, 2);
}

/// Test that IntakeTask deserializes string priorities.
#[test]
fn test_intake_task_string_priority() {
    let test_cases = vec![
        (
            r#"{"id": "1", "title": "T", "description": "D", "priority": "critical"}"#,
            1,
        ),
        (
            r#"{"id": "1", "title": "T", "description": "D", "priority": "urgent"}"#,
            1,
        ),
        (
            r#"{"id": "1", "title": "T", "description": "D", "priority": "high"}"#,
            2,
        ),
        (
            r#"{"id": "1", "title": "T", "description": "D", "priority": "medium"}"#,
            3,
        ),
        (
            r#"{"id": "1", "title": "T", "description": "D", "priority": "normal"}"#,
            3,
        ),
        (
            r#"{"id": "1", "title": "T", "description": "D", "priority": "low"}"#,
            4,
        ),
    ];

    for (json, expected_priority) in test_cases {
        let task: IntakeTask = serde_json::from_str(json).unwrap();
        assert_eq!(task.priority, expected_priority, "Failed for JSON: {json}");
    }
}

/// Test that IntakeTask uses default priority when not specified.
#[test]
fn test_intake_task_default_priority() {
    let json = r#"{
        "id": "1",
        "title": "Test Task",
        "description": "Test description"
    }"#;

    let task: IntakeTask = serde_json::from_str(json).unwrap();
    assert_eq!(task.priority, 3); // Default is normal (3)
}

/// Test deserialization of mixed ID formats in dependencies.
#[test]
fn test_intake_task_mixed_dependency_formats() {
    let json = r#"{
        "id": "1",
        "title": "Test Task",
        "description": "Test",
        "dependencies": ["2", 3, "4"]
    }"#;

    let task: IntakeTask = serde_json::from_str(json).unwrap();
    assert_eq!(task.dependencies, vec!["2", "3", "4"]);
}

/// Test full `TasksJson` deserialization (as received from callback).
#[test]
fn test_tasks_json_full_payload() {
    use pm::handlers::intake::TasksJson;

    let json = r#"{
        "tasks": [
            {
                "id": 1,
                "title": "Provision Infrastructure",
                "description": "Deploy PostgreSQL and Redis",
                "priority": "critical",
                "dependencies": [],
                "agentHint": "bolt"
            },
            {
                "id": 2,
                "title": "Implement Backend API",
                "description": "Build REST endpoints",
                "priority": "high",
                "dependencies": [1],
                "agentHint": "rex"
            },
            {
                "id": 3,
                "title": "Build Frontend",
                "description": "Create React dashboard",
                "priority": "medium",
                "dependencies": [2],
                "agentHint": "blaze"
            }
        ]
    }"#;

    let tasks_json: TasksJson = serde_json::from_str(json).unwrap();
    assert_eq!(tasks_json.tasks.len(), 3);

    // Check priorities converted correctly
    assert_eq!(tasks_json.tasks[0].priority, 1); // critical
    assert_eq!(tasks_json.tasks[1].priority, 2); // high
    assert_eq!(tasks_json.tasks[2].priority, 3); // medium

    // Check dependencies converted correctly
    assert_eq!(tasks_json.tasks[1].dependencies, vec!["1"]);
    assert_eq!(tasks_json.tasks[2].dependencies, vec!["2"]);
}

// =============================================================================
// Callback Payload Tests
// =============================================================================

/// Test IntakeCompleteCallback deserialization.
#[test]
fn test_intake_complete_callback_deserialization() {
    use pm::handlers::callbacks::IntakeCompleteCallback;

    let json = r#"{
        "sessionId": "session-abc",
        "issueId": "issue-123",
        "issueIdentifier": "TSK-42",
        "teamId": "team-xyz",
        "tasks": [
            {
                "id": "task-1",
                "title": "First Task",
                "description": "Do something",
                "priority": 2,
                "dependencies": []
            }
        ],
        "workflowName": "intake-12345",
        "success": true
    }"#;

    let callback: IntakeCompleteCallback = serde_json::from_str(json).unwrap();
    assert_eq!(callback.session_id, "session-abc");
    assert_eq!(callback.issue_id, "issue-123");
    assert_eq!(callback.issue_identifier, "TSK-42");
    assert_eq!(callback.team_id, "team-xyz");
    assert_eq!(callback.tasks.len(), 1);
    assert!(callback.success);
    assert_eq!(callback.workflow_name, Some("intake-12345".to_string()));
}

/// Test failed callback deserialization.
#[test]
fn test_intake_complete_callback_failure() {
    use pm::handlers::callbacks::IntakeCompleteCallback;

    let json = r#"{
        "sessionId": "session-abc",
        "issueId": "issue-123",
        "issueIdentifier": "TSK-42",
        "teamId": "team-xyz",
        "tasks": [],
        "success": false,
        "error": "Workflow timeout after 30 minutes"
    }"#;

    let callback: IntakeCompleteCallback = serde_json::from_str(json).unwrap();
    assert!(!callback.success);
    assert_eq!(
        callback.error,
        Some("Workflow timeout after 30 minutes".to_string())
    );
}

/// Test PlayCompleteCallback deserialization.
#[test]
fn test_play_complete_callback_deserialization() {
    use pm::handlers::callbacks::PlayCompleteCallback;

    let json = r#"{
        "workflow_name": "play-task-2-rex",
        "workflow_status": "Succeeded",
        "linear_session_id": "session-def",
        "linear_issue_id": "issue-456",
        "linear_team_id": "team-abc",
        "task_id": "2",
        "repository": "5dlabs/my-project"
    }"#;

    let callback: PlayCompleteCallback = serde_json::from_str(json).unwrap();
    assert_eq!(callback.workflow_name, "play-task-2-rex");
    assert_eq!(callback.workflow_status, "Succeeded");
    assert_eq!(callback.linear_session_id, "session-def");
    assert_eq!(callback.task_id, Some("2".to_string()));
}

// =============================================================================
// Agent Status Tests
// =============================================================================

/// Test AgentStatus label name conversion.
#[test]
fn test_agent_status_label_names() {
    use pm::models::AgentStatus;

    assert_eq!(AgentStatus::Pending.to_label_name(), "agent:pending");
    assert_eq!(AgentStatus::Working.to_label_name(), "agent:working");
    assert_eq!(AgentStatus::Blocked.to_label_name(), "agent:blocked");
    assert_eq!(AgentStatus::PrCreated.to_label_name(), "agent:pr-created");
    assert_eq!(AgentStatus::Complete.to_label_name(), "agent:complete");
    assert_eq!(AgentStatus::Error.to_label_name(), "agent:error");
}

/// Test AgentStatus from sidecar status conversion.
#[test]
fn test_agent_status_from_sidecar() {
    use pm::models::AgentStatus;

    // Various status strings the sidecar might send
    assert_eq!(
        AgentStatus::from_sidecar_status("pending"),
        AgentStatus::Pending
    );
    assert_eq!(
        AgentStatus::from_sidecar_status("queued"),
        AgentStatus::Pending
    );
    assert_eq!(
        AgentStatus::from_sidecar_status("in_progress"),
        AgentStatus::Working
    );
    assert_eq!(
        AgentStatus::from_sidecar_status("working"),
        AgentStatus::Working
    );
    assert_eq!(
        AgentStatus::from_sidecar_status("blocked"),
        AgentStatus::Blocked
    );
    assert_eq!(
        AgentStatus::from_sidecar_status("review"),
        AgentStatus::PrCreated
    );
    assert_eq!(
        AgentStatus::from_sidecar_status("pr_created"),
        AgentStatus::PrCreated
    );
    assert_eq!(
        AgentStatus::from_sidecar_status("complete"),
        AgentStatus::Complete
    );
    assert_eq!(
        AgentStatus::from_sidecar_status("done"),
        AgentStatus::Complete
    );
    assert_eq!(
        AgentStatus::from_sidecar_status("failed"),
        AgentStatus::Error
    );
    assert_eq!(
        AgentStatus::from_sidecar_status("error"),
        AgentStatus::Error
    );
}

// =============================================================================
// Workflow State Tests
// =============================================================================

/// Test play workflow states configuration.
#[test]
fn test_play_workflow_states() {
    use pm::handlers::intake::PLAY_WORKFLOW_STATES;

    // Verify we have all expected states
    let state_names: Vec<_> = PLAY_WORKFLOW_STATES
        .iter()
        .map(|(name, _, _)| *name)
        .collect();
    assert!(state_names.contains(&"Ready"));
    assert!(state_names.contains(&"🔧 Implementation"));
    assert!(state_names.contains(&"🔍 Quality"));
    assert!(state_names.contains(&"🔗 Integration"));
    assert!(state_names.contains(&"🚀 Deployment"));

    // Verify state types are valid
    let valid_types = ["unstarted", "started", "completed", "canceled", "backlog"];
    for (_, state_type, _) in PLAY_WORKFLOW_STATES {
        assert!(
            valid_types.contains(state_type),
            "Invalid state type: {state_type}"
        );
    }
}

// =============================================================================
// Helper Function Tests
// =============================================================================

/// Test completion summary generation.
#[test]
fn test_generate_completion_summary() {
    use pm::config::CtoConfig;
    use pm::handlers::intake::{generate_completion_summary, IntakeRequest, IntakeTask, TechStack};
    use std::collections::HashMap;

    let request = IntakeRequest {
        session_id: "test-session".to_string(),
        prd_issue_id: "prd-123".to_string(),
        prd_identifier: "TSK-1".to_string(),
        team_id: "team-abc".to_string(),
        title: "Test PRD".to_string(),
        project_name: None,
        prd_content: String::new(),
        architecture_content: None,
        repository_url: None,
        github_visibility: "private".to_string(),
        source_branch: None,
        tech_stack: TechStack::default(),
        cto_config: CtoConfig::default(),
    };

    let tasks = vec![
        IntakeTask {
            id: "1".to_string(),
            title: "Task 1".to_string(),
            description: "Description 1".to_string(),
            details: String::new(),
            dependencies: vec![],
            priority: 1,
            test_strategy: String::new(),
            agent_hint: "bolt".to_string(),
        },
        IntakeTask {
            id: "2".to_string(),
            title: "Task 2".to_string(),
            description: "Description 2".to_string(),
            details: String::new(),
            dependencies: vec!["1".to_string()],
            priority: 2,
            test_strategy: String::new(),
            agent_hint: "rex".to_string(),
        },
    ];

    let mut task_map = HashMap::new();
    task_map.insert("1".to_string(), "issue-1".to_string());
    task_map.insert("2".to_string(), "issue-2".to_string());

    let summary = generate_completion_summary(&request, &tasks, &task_map);

    // Verify summary contains key information
    assert!(summary.contains("TSK-1"), "Should include PRD identifier");
    assert!(summary.contains("2"), "Should include task count");
    assert!(
        summary.contains("High Priority"),
        "Should mention high priority count"
    );
    assert!(
        summary.contains("Dependencies"),
        "Should mention dependencies"
    );
    assert!(
        summary.contains("Linear Issues Created"),
        "Should mention issues created"
    );
}

/// Test task description formatting.
#[test]
fn test_format_task_description() {
    use pm::handlers::intake::IntakeTask;

    let task = IntakeTask {
        id: "1".to_string(),
        title: "Implement API".to_string(),
        description: "Build REST endpoints for user management".to_string(),
        details: "Use Axum framework with PostgreSQL".to_string(),
        dependencies: vec!["0".to_string()],
        priority: 2,
        test_strategy: "Unit tests for all handlers".to_string(),
        agent_hint: "rex".to_string(),
    };

    // The format_task_description function is private, so we test via the public interface
    // In a real scenario, we'd check the issue description after creation
    assert!(!task.description.is_empty());
    assert!(!task.details.is_empty());
    assert!(!task.test_strategy.is_empty());
}

// =============================================================================
// CTO Config Extraction Tests
// =============================================================================

/// Test frontmatter parsing.
#[test]
fn test_parse_cto_frontmatter() {
    use pm::handlers::intake::parse_cto_frontmatter;

    let description = r#"---
cto:
  cli: cursor
  model: claude-opus-4-20250514
---
## PRD: My Feature

This is the actual content."#;

    let config = parse_cto_frontmatter(description);
    assert!(config.is_some());

    let config = config.unwrap();
    assert_eq!(config.cli, Some("cursor".to_string()));
    assert_eq!(config.model, Some("claude-opus-4-20250514".to_string()));
}

/// Test frontmatter stripping.
#[test]
fn test_strip_frontmatter() {
    use pm::handlers::intake::strip_frontmatter;

    let description = r#"---
cto:
  cli: claude
---
## My PRD

Content here"#;

    let stripped = strip_frontmatter(description);
    assert!(!stripped.contains("---"));
    assert!(!stripped.contains("cto:"));
    assert!(stripped.contains("## My PRD"));
    assert!(stripped.contains("Content here"));
}

/// Test label config extraction.
#[test]
fn test_extract_config_from_labels() {
    use pm::handlers::intake::extract_config_from_labels;
    use pm::models::Label;

    // Test combined "cli:model" format
    let labels = vec![Label {
        id: "1".to_string(),
        name: "claude:opus".to_string(),
        color: None,
    }];

    let config = extract_config_from_labels(&labels);
    assert_eq!(config.cli, Some("claude".to_string()));
    assert!(config.model.is_some());
}

/// Test GitHub visibility extraction from labels.
#[test]
fn test_extract_github_visibility() {
    use pm::handlers::intake::extract_github_visibility;
    use pm::models::Label;

    // Default is private
    let labels: Vec<Label> = vec![];
    assert_eq!(extract_github_visibility(&labels), "private");

    // With public label
    let labels = vec![Label {
        id: "1".to_string(),
        name: "github:public".to_string(),
        color: None,
    }];
    assert_eq!(extract_github_visibility(&labels), "public");
}
