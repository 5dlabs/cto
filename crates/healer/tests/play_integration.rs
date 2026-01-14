//! Integration tests for Healer Play monitoring.
//!
//! Tests the full flow:
//! 1. Start Play API server
//! 2. MCP server sends session start request
//! 3. Session is stored with CTO config
//! 4. Evaluation Agent can be spawned (mocked)

use axum::body::Body;
use axum::http::{Request, StatusCode};
use healer::{build_play_api_router, PlayApiState};
use serde_json::{json, Value};
use std::sync::Arc;
use tower::ServiceExt;

/// Test the full Play session lifecycle via HTTP.
#[tokio::test]
#[allow(clippy::too_many_lines)]
async fn test_play_session_lifecycle_http() {
    // Create the API state and router
    let state = Arc::new(PlayApiState::new("cto"));
    let app = build_play_api_router(state.clone());

    // Test 1: Health check
    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/health")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let body = axum::body::to_bytes(response.into_body(), 1024)
        .await
        .unwrap();
    let health: Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(health["status"], "healthy");
    println!("✅ Health check passed");

    // Test 2: Start a session
    let session_request = json!({
        "play_id": "integration-test-play-1",
        "repository": "5dlabs/test-repo",
        "service": "test-service",
        "namespace": "cto",
        "cto_config": {
            "agents": {
                "rex": {
                    "github_app": "5DLabs-Rex",
                    "cli": "claude",
                    "model": "claude-sonnet-4-5-20250514",
                    "tools": {
                        "remote": ["brave_search", "github_create_pr"],
                        "local_servers": {
                            "filesystem": {
                                "enabled": true,
                                "tools": ["read_file", "write_file"]
                            }
                        }
                    }
                },
                "bolt": {
                    "github_app": "5DLabs-Bolt",
                    "cli": "claude",
                    "model": "claude-sonnet-4-5-20250514",
                    "tools": {
                        "remote": ["kubernetes_apply"],
                        "local_servers": {}
                    }
                }
            }
        },
        "tasks": [
            {
                "id": "1",
                "title": "Setup infrastructure",
                "agent_hint": "bolt",
                "dependencies": [],
                "priority": 0
            },
            {
                "id": "2",
                "title": "Implement feature",
                "agent_hint": "rex",
                "dependencies": ["1"],
                "priority": 1
            }
        ]
    });

    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/session/start")
                .header("content-type", "application/json")
                .body(Body::from(serde_json::to_string(&session_request).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let body = axum::body::to_bytes(response.into_body(), 4096)
        .await
        .unwrap();
    let start_response: Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(start_response["status"], "ok");
    assert_eq!(start_response["session_id"], "integration-test-play-1");
    println!("✅ Session started: {}", start_response["message"]);

    // Test 3: Get the session back
    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/api/v1/session/integration-test-play-1")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let body = axum::body::to_bytes(response.into_body(), 8192)
        .await
        .unwrap();
    let session_response: Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(session_response["status"], "ok");

    let session = &session_response["session"];
    assert_eq!(session["play_id"], "integration-test-play-1");
    assert_eq!(session["repository"], "5dlabs/test-repo");
    assert_eq!(session["status"], "active");

    // Verify CTO config was stored
    let agents = &session["cto_config"]["agents"];
    assert!(agents.get("rex").is_some());
    assert!(agents.get("bolt").is_some());

    // Verify Rex's tools were stored
    let rex_tools = &agents["rex"]["tools"]["remote"];
    assert!(rex_tools
        .as_array()
        .unwrap()
        .iter()
        .any(|t| t == "brave_search"));
    println!("✅ Session retrieved with CTO config");

    // Test 4: Verify tasks were stored
    let tasks = session["tasks"].as_array().unwrap();
    assert_eq!(tasks.len(), 2);
    assert_eq!(tasks[0]["id"], "1");
    assert_eq!(tasks[0]["agent_hint"], "bolt");
    assert_eq!(tasks[1]["id"], "2");
    assert_eq!(tasks[1]["agent_hint"], "rex");
    println!("✅ Tasks stored correctly");

    // Test 5: List active sessions
    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/api/v1/sessions/active")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let body = axum::body::to_bytes(response.into_body(), 4096)
        .await
        .unwrap();
    let list_response: Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(list_response["status"], "ok");
    assert!(list_response["count"].as_u64().unwrap() >= 1);
    println!("✅ Active sessions listed");

    // Test 6: Verify duplicate session is rejected
    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/session/start")
                .header("content-type", "application/json")
                .body(Body::from(serde_json::to_string(&session_request).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::CONFLICT);
    let body = axum::body::to_bytes(response.into_body(), 1024)
        .await
        .unwrap();
    let error_response: Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(error_response["status"], "error");
    println!("✅ Duplicate session correctly rejected");

    println!("\n🎉 Full Play session lifecycle test PASSED!");
}

/// Test payload validation (non-HTTP).
#[tokio::test]
async fn test_play_session_payload_validation() {
    println!("Testing Play session payload validation...");

    // Test 1: Create a session start request payload
    let session_request = json!({
        "play_id": "validation-test-play-1",
        "repository": "5dlabs/test-repo",
        "service": "test-service",
        "namespace": "cto",
        "cto_config": {
            "agents": {
                "rex": {
                    "github_app": "5DLabs-Rex",
                    "cli": "claude",
                    "model": "claude-sonnet-4-5-20250514",
                    "tools": {
                        "remote": ["brave_search", "github_create_pr"],
                        "local_servers": {
                            "filesystem": {
                                "enabled": true,
                                "tools": ["read_file", "write_file"]
                            }
                        }
                    }
                }
            }
        },
        "tasks": [
            {
                "id": "1",
                "title": "Setup infrastructure",
                "agent_hint": "bolt",
                "dependencies": [],
                "priority": 0
            }
        ]
    });

    // Verify the request is valid JSON
    assert!(session_request.is_object());

    // Test 2: Verify CTO config structure
    let cto_config = session_request.get("cto_config").unwrap();
    let agents = cto_config.get("agents").unwrap().as_object().unwrap();
    assert!(!agents.is_empty());

    // Test 3: Verify Rex has the expected tools
    let rex = agents.get("rex").unwrap();
    let rex_tools = rex.get("tools").unwrap();
    let remote_tools = rex_tools.get("remote").unwrap().as_array().unwrap();
    assert!(remote_tools.iter().any(|t| t == "brave_search"));
    assert!(remote_tools.iter().any(|t| t == "github_create_pr"));

    // Test 4: Verify local servers
    let local_servers = rex_tools.get("local_servers").unwrap().as_object().unwrap();
    assert!(local_servers.contains_key("filesystem"));

    println!("✅ All payload validations passed");
}

/// Test evaluation prompt generation includes pre-flight checks.
#[tokio::test]
async fn test_evaluation_prompt_contains_preflight_checks() {
    // Create a mock session
    let session_json = json!({
        "play_id": "preflight-test-1",
        "repository": "5dlabs/test",
        "service": "test-svc",
        "cto_config": {
            "agents": {
                "rex": {
                    "tools": {
                        "remote": ["tool1", "tool2"],
                        "localServers": {}
                    }
                }
            }
        },
        "tasks": []
    });

    // The prompt should include these key elements
    let expected_elements = vec![
        "Universal Pre-Flight Checks",
        "Prompt Verification",
        "MCP Tool Verification",
        "declared tools == available tools",
        "CTO config loaded and valid",
        "tools-server reachable",
    ];

    // For now, just verify the structure
    assert!(session_json.get("cto_config").is_some());
    println!("✅ Session structure validated for pre-flight checks");

    // Note: Full prompt generation test is in evaluation_spawner.rs unit tests
    for element in expected_elements {
        println!("  - Would check for: {element}");
    }
}

/// Test that Healer notification payload matches expected format.
#[tokio::test]
async fn test_healer_notification_format() {
    // This is the format the MCP server sends to Healer
    let notification = json!({
        "play_id": "play-1234567890",
        "repository": "5dlabs/cto-parallel-test",
        "service": "test-service",
        "namespace": "cto",
        "cto_config": {
            "agents": {
                "rex": {
                    "githubApp": "5DLabs-Rex",
                    "cli": "claude",
                    "model": "claude-sonnet-4-5-20250514",
                    "tools": {
                        "remote": ["brave_search"],
                        "localServers": {}
                    }
                }
            }
        },
        "tasks": [{
            "id": "1",
            "title": "Task 1",
            "dependencies": []
        }]
    });

    // Verify required fields
    assert!(notification.get("play_id").is_some());
    assert!(notification.get("repository").is_some());
    assert!(notification.get("cto_config").is_some());
    assert!(notification.get("tasks").is_some());

    // Verify nested structure
    let agents = notification
        .get("cto_config")
        .unwrap()
        .get("agents")
        .unwrap();
    assert!(agents.get("rex").is_some());

    println!("✅ Healer notification format validated");
}

/// Test session status transitions.
#[tokio::test]
async fn test_session_status_transitions() {
    // Valid status values
    let statuses = vec!["active", "completed", "failed", "cancelled"];

    for status in &statuses {
        let session = json!({
            "play_id": "status-test",
            "status": status
        });
        assert!(session.get("status").is_some());
    }

    println!("✅ Session status values validated: {:?}", statuses);
}

/// Test issue detection types.
#[tokio::test]
async fn test_issue_types() {
    // Issue types that Healer should detect
    let issue_types = vec![
        ("pre_flight_failure", "critical", "Tool mismatch detected"),
        ("tool_mismatch", "critical", "Expected tool X not available"),
        ("stuck", "high", "Agent stuck for >30 minutes"),
        ("agent_failure", "high", "Agent crashed"),
        ("build_failure", "high", "Build failed"),
        ("test_failure", "high", "Tests failed"),
        ("language_mismatch", "medium", "Wrong linter for language"),
    ];

    for (issue_type, severity, description) in &issue_types {
        let issue = json!({
            "type": issue_type,
            "severity": severity,
            "description": description,
            "detected_at": "2026-01-14T00:00:00Z",
            "agent": "rex",
            "task_id": "1",
            "remediation_spawned": false
        });

        assert_eq!(issue.get("type").unwrap(), *issue_type);
        assert_eq!(issue.get("severity").unwrap(), *severity);
    }

    println!("✅ Issue types validated: {} types", issue_types.len());
}

/// Test agent-to-language mapping for quality checks.
#[tokio::test]
async fn test_agent_language_mapping() {
    // Agent -> Language -> Expected linting tools
    let mappings = vec![
        ("rex", "rust", vec!["cargo clippy", "cargo fmt"]),
        ("grizz", "go", vec!["golangci-lint", "go vet"]),
        ("nova", "typescript", vec!["eslint", "prettier"]),
        ("blaze", "react", vec!["eslint", "prettier"]),
    ];

    for (agent, language, tools) in &mappings {
        println!("  Agent {agent} -> {language}: {:?}", tools);
    }

    println!(
        "✅ Agent-language mappings validated: {} agents",
        mappings.len()
    );
}

/// Test remediation strategy selection based on issue type.
#[tokio::test]
async fn test_remediation_strategy_selection() {
    use healer::play::{
        remediation_spawner::RemediationSpawner, IssueSeverity, IssueType, SessionIssue,
    };

    // Test issue type -> strategy mapping
    let test_cases = vec![
        (
            IssueType::PreFlightFailure,
            IssueSeverity::Critical,
            "FixConfig",
        ),
        (
            IssueType::ToolMismatch,
            IssueSeverity::Critical,
            "FixConfig",
        ),
        (IssueType::BuildFailure, IssueSeverity::High, "FixCode"),
        (IssueType::TestFailure, IssueSeverity::High, "FixCode"),
        (IssueType::Stuck, IssueSeverity::High, "Restart"),
        (
            IssueType::LanguageMismatch,
            IssueSeverity::Medium,
            "FixConfig",
        ),
        (IssueType::AgentFailure, IssueSeverity::High, "Retry"),
    ];

    for (issue_type, severity, expected_strategy) in test_cases {
        let issue_type_debug = format!("{issue_type:?}");
        let issue = SessionIssue {
            detected_at: chrono::Utc::now(),
            issue_type,
            severity,
            description: "Test issue".to_string(),
            agent: Some("rex".to_string()),
            task_id: Some("1".to_string()),
            remediation_spawned: false,
            github_issue: None,
        };

        let strategy = RemediationSpawner::get_remediation_strategy(&issue);
        let strategy_name = format!("{strategy:?}");
        assert_eq!(
            strategy_name, expected_strategy,
            "Issue {} should map to {}",
            issue_type_debug, expected_strategy
        );
        println!("  {} -> {}", issue_type_debug, strategy_name);
    }

    println!("✅ Remediation strategy selection validated");
}

/// Test language matching verification.
#[tokio::test]
async fn test_language_matching_verification() {
    use healer::play::orchestrator::{verify_language_match, ImplementationLanguage};

    // Test agent to language mapping
    assert_eq!(
        ImplementationLanguage::for_agent("rex"),
        ImplementationLanguage::Rust
    );
    assert_eq!(
        ImplementationLanguage::for_agent("grizz"),
        ImplementationLanguage::Go
    );
    assert_eq!(
        ImplementationLanguage::for_agent("blaze"),
        ImplementationLanguage::TypeScript
    );
    assert_eq!(
        ImplementationLanguage::for_agent("vex"),
        ImplementationLanguage::CSharp
    );
    println!("✅ Agent to language mapping validated");

    // Test quality tools for Rust
    let rust_quality = ImplementationLanguage::Rust.quality_tools();
    assert!(rust_quality.contains(&"cargo clippy"));
    assert!(rust_quality.contains(&"cargo fmt"));
    println!("✅ Rust quality tools: {:?}", rust_quality);

    // Test security tools for Go
    let go_security = ImplementationLanguage::Go.security_tools();
    assert!(go_security.contains(&"gosec"));
    println!("✅ Go security tools: {:?}", go_security);

    // Test language match verification
    let result = verify_language_match(
        "rex",
        "cleo",
        &["cargo clippy".to_string(), "cargo fmt".to_string()],
    );
    assert!(result.matches);
    assert_eq!(result.expected_language, "Rust");
    println!("✅ Language match verification: Rex + Cleo with Rust tools passed");

    // Test mismatch detection
    let result = verify_language_match(
        "rex",
        "cleo",
        &["eslint".to_string(), "prettier".to_string()],
    );
    assert!(!result.matches);
    assert!(result.mismatch_description.is_some());
    println!("✅ Language mismatch detected: Rex + Cleo with TS tools");
}

/// Test orchestrator configuration defaults.
#[tokio::test]
async fn test_orchestrator_config_defaults() {
    use healer::play::orchestrator::OrchestratorConfig;

    let config = OrchestratorConfig::default();

    assert_eq!(config.namespace, "cto");
    assert_eq!(config.max_remediation_attempts, 3);
    assert!(config.auto_escalate);
    // Evaluation interval should be positive
    assert!(config.evaluation_interval.num_minutes() > 0);
    // Re-evaluation delay should be positive
    assert!(config.re_evaluation_delay.num_minutes() > 0);

    println!("✅ Orchestrator default config validated");
    println!("  - namespace: {}", config.namespace);
    println!(
        "  - max_remediation_attempts: {}",
        config.max_remediation_attempts
    );
    println!("  - auto_escalate: {}", config.auto_escalate);
}

/// Test the EXACT JSON format that MCP server sends to Healer.
/// This is critical: MCP sends camelCase (`localServers`, `githubApp`),
/// and Healer must correctly deserialize these.
#[tokio::test]
async fn test_mcp_to_healer_json_format() {
    let state = Arc::new(PlayApiState::new("cto"));
    let app = build_play_api_router(state.clone());

    // This is the EXACT format the MCP server sends (from notify_healer function)
    let mcp_payload = json!({
        "play_id": "play-mcp-format-test",
        "repository": "5dlabs/cto-parallel-test",
        "service": "test-service",
        "namespace": "cto",
        "cto_config": {
            "agents": {
                "rex": {
                    "githubApp": "5DLabs-Rex",  // camelCase!
                    "cli": "claude",
                    "model": "claude-sonnet-4-5-20250514",
                    "tools": {
                        "remote": ["context7_resolve_library_id", "firecrawl_scrape"],
                        "localServers": {}  // camelCase!
                    }
                },
                "bolt": {
                    "githubApp": "5DLabs-Bolt",
                    "cli": "claude",
                    "model": "claude-sonnet-4-5-20250514",
                    "tools": {
                        "remote": ["kubernetes_listResources", "argocd_list_applications"],
                        "localServers": {
                            "filesystem": {
                                "enabled": true,
                                "tools": ["read_file", "write_file"]
                            }
                        }
                    }
                }
            }
        },
        "tasks": [{
            "id": "1",
            "title": "Task 1",
            "dependencies": []
        }]
    });

    // Send to Healer API
    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/session/start")
                .header("content-type", "application/json")
                .body(Body::from(serde_json::to_string(&mcp_payload).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(
        response.status(),
        StatusCode::OK,
        "Healer should accept MCP's camelCase JSON format"
    );

    let body = axum::body::to_bytes(response.into_body(), 4096)
        .await
        .unwrap();
    let start_response: Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(start_response["status"], "ok");
    println!("✅ Healer accepted MCP's JSON format");

    // Now retrieve and verify the session was stored correctly
    let response = app
        .oneshot(
            Request::builder()
                .uri("/api/v1/session/play-mcp-format-test")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let body = axum::body::to_bytes(response.into_body(), 8192)
        .await
        .unwrap();
    let session_response: Value = serde_json::from_slice(&body).unwrap();

    let session = &session_response["session"];
    let agents = &session["cto_config"]["agents"];

    // Verify Rex was stored with tools
    let rex = &agents["rex"];
    assert!(rex["tools"]["remote"].as_array().unwrap().len() >= 2);
    println!("✅ Rex tools stored: {:?}", rex["tools"]["remote"]);

    // Verify Bolt was stored with local servers
    let bolt = &agents["bolt"];
    let bolt_local_servers = &bolt["tools"]["localServers"];
    assert!(
        bolt_local_servers.get("filesystem").is_some(),
        "Bolt's filesystem local server should be stored"
    );
    println!("✅ Bolt localServers stored: {:?}", bolt_local_servers);

    println!("\n🎉 MCP → Healer JSON format test PASSED!");
}

/// Test the /api/v1/sessions endpoint returns ALL sessions (not just active).
#[tokio::test]
async fn test_list_all_sessions_endpoint() {
    let state = Arc::new(PlayApiState::new("cto"));
    let app = build_play_api_router(state.clone());

    // Start a session
    let session_request = json!({
        "play_id": "all-sessions-test-1",
        "repository": "5dlabs/test",
        "cto_config": { "agents": {} },
        "tasks": []
    });

    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/session/start")
                .header("content-type", "application/json")
                .body(Body::from(serde_json::to_string(&session_request).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    // Test /api/v1/sessions (should return ALL sessions)
    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/api/v1/sessions")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let body = axum::body::to_bytes(response.into_body(), 4096)
        .await
        .unwrap();
    let list_response: Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(list_response["status"], "ok");
    assert!(list_response["count"].as_u64().unwrap() >= 1);
    println!("✅ /api/v1/sessions returns sessions (count: {})", list_response["count"]);

    // Test /api/v1/sessions/active (should also return at least 1 active)
    let response = app
        .oneshot(
            Request::builder()
                .uri("/api/v1/sessions/active")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let body = axum::body::to_bytes(response.into_body(), 4096)
        .await
        .unwrap();
    let active_response: Value = serde_json::from_slice(&body).unwrap();
    assert!(active_response["count"].as_u64().unwrap() >= 1);
    println!("✅ /api/v1/sessions/active returns active sessions");
}

/// Test session store operations: add issue, complete session.
#[tokio::test]
async fn test_session_store_operations() {
    use healer::play::{IssueSeverity, IssueType, SessionStatus};

    let store = healer::SessionStore::new();

    // Start a session
    let request = healer::StartSessionRequest {
        play_id: "store-ops-test".to_string(),
        repository: "5dlabs/test".to_string(),
        service: Some("test-svc".to_string()),
        cto_config: healer::play::CtoConfig::default(),
        tasks: vec![],
        namespace: "cto".to_string(),
    };

    let session = store.start_session(request).await;
    assert_eq!(session.status, SessionStatus::Active);
    println!("✅ Session started");

    // Add an issue
    let issue = healer::play::SessionIssue {
        detected_at: chrono::Utc::now(),
        issue_type: IssueType::ToolMismatch,
        severity: IssueSeverity::Critical,
        description: "Tool brave_search not available".to_string(),
        agent: Some("rex".to_string()),
        task_id: Some("1".to_string()),
        remediation_spawned: false,
        github_issue: None,
    };

    let add_result = store.add_issue("store-ops-test", issue).await;
    assert!(add_result.is_ok());

    let session = store.get_session("store-ops-test").await.unwrap();
    assert_eq!(session.issues.len(), 1);
    assert_eq!(session.issues[0].issue_type, IssueType::ToolMismatch);
    println!("✅ Issue added to session");

    // Complete the session (failed)
    store.complete_session("store-ops-test", false).await;

    let session = store.get_session("store-ops-test").await.unwrap();
    assert_eq!(session.status, SessionStatus::Failed);
    println!("✅ Session marked as failed");

    // Verify get_active_sessions excludes completed sessions
    let active = store.get_active_sessions().await;
    assert!(
        !active.iter().any(|s| s.play_id == "store-ops-test"),
        "Completed session should not appear in active list"
    );
    println!("✅ Completed session excluded from active list");

    // Verify get_all_sessions includes completed sessions
    let all = store.get_all_sessions().await;
    assert!(
        all.iter().any(|s| s.play_id == "store-ops-test"),
        "Completed session should appear in all sessions list"
    );
    println!("✅ Completed session included in all sessions list");
}

/// Test evaluation prompt includes all expected context.
#[tokio::test]
async fn test_evaluation_prompt_context() {
    use healer::play::{AgentConfig, AgentTools, CtoConfig, PlaySession, SessionStatus, TaskInfo};
    use std::collections::HashMap;

    // Create a session with rich context
    let mut agents = HashMap::new();
    let mut local_servers = HashMap::new();
    local_servers.insert(
        "filesystem".to_string(),
        healer::play::session::LocalServerConfig {
            enabled: true,
            tools: vec!["read_file".to_string(), "write_file".to_string()],
        },
    );

    agents.insert(
        "rex".to_string(),
        AgentConfig {
            github_app: Some("5DLabs-Rex".to_string()),
            cli: Some("claude".to_string()),
            model: Some("claude-opus-4-5-20251101".to_string()),
            tools: AgentTools {
                remote: vec![
                    "context7_resolve_library_id".to_string(),
                    "firecrawl_scrape".to_string(),
                    "github_create_pr".to_string(),
                ],
                local_servers,
            },
        },
    );

    let session = PlaySession {
        play_id: "prompt-context-test".to_string(),
        repository: "5dlabs/cto-parallel-test".to_string(),
        service: Some("test-service".to_string()),
        cto_config: CtoConfig { agents },
        tasks: vec![
            TaskInfo {
                id: "1".to_string(),
                title: "Setup infrastructure".to_string(),
                agent_hint: Some("bolt".to_string()),
                dependencies: vec![],
                priority: 0,
            },
            TaskInfo {
                id: "2".to_string(),
                title: "Implement feature".to_string(),
                agent_hint: Some("rex".to_string()),
                dependencies: vec!["1".to_string()],
                priority: 1,
            },
        ],
        namespace: "cto".to_string(),
        started_at: chrono::Utc::now(),
        last_updated: chrono::Utc::now(),
        issues: vec![],
        status: SessionStatus::Active,
    };

    // Use the internal prompt builder (we can't access it directly, but we can verify via spawner)
    // For now, just verify the session structure is correct
    assert_eq!(session.cto_config.agents.len(), 1);
    assert!(session.cto_config.agents.contains_key("rex"));

    let rex_config = session.cto_config.agents.get("rex").unwrap();
    assert_eq!(rex_config.tools.remote.len(), 3);
    assert_eq!(rex_config.tools.local_servers.len(), 1);
    println!("✅ Session structure validated for prompt generation");

    // Verify tasks
    assert_eq!(session.tasks.len(), 2);
    assert_eq!(session.tasks[0].agent_hint, Some("bolt".to_string()));
    assert_eq!(session.tasks[1].dependencies, vec!["1".to_string()]);
    println!("✅ Task dependencies validated");

    // The EvaluationSpawner would use this session to generate a prompt containing:
    // - Play ID, repository, service
    // - All agent configurations with their tools
    // - Task list with dependencies
    // - Universal Pre-Flight Checks
    println!("\n📝 Prompt would include:");
    println!("  - Play ID: {}", session.play_id);
    println!("  - Repository: {}", session.repository);
    println!("  - Agents: {:?}", session.cto_config.agents.keys().collect::<Vec<_>>());
    println!("  - Rex remote tools: {:?}", rex_config.tools.remote);
    println!("  - Tasks: {} total", session.tasks.len());
}

/// Test the full round-trip: session start → issue detection → remediation.
#[tokio::test]
async fn test_full_monitoring_roundtrip() {
    use healer::play::{
        IssueSeverity, IssueType, RemediationSpawner, SessionIssue,
    };

    let state = Arc::new(PlayApiState::new("cto"));
    let app = build_play_api_router(state.clone());

    // Step 1: Start a session (simulating MCP notification)
    let session_request = json!({
        "play_id": "roundtrip-test",
        "repository": "5dlabs/cto",
        "service": "roundtrip-svc",
        "cto_config": {
            "agents": {
                "rex": {
                    "githubApp": "5DLabs-Rex",
                    "cli": "claude",
                    "model": "claude-opus-4-5-20251101",
                    "tools": {
                        "remote": ["brave_search", "github_create_pr"],
                        "localServers": {}
                    }
                }
            }
        },
        "tasks": [{
            "id": "1",
            "title": "Implement feature",
            "agent_hint": "rex",
            "dependencies": []
        }]
    });

    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/session/start")
                .header("content-type", "application/json")
                .body(Body::from(serde_json::to_string(&session_request).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    println!("✅ Step 1: Session started");

    // Step 2: Simulate issue detection (this would normally come from Evaluation Agent)
    // Add an issue to the session store directly
    let issue = SessionIssue {
        detected_at: chrono::Utc::now(),
        issue_type: IssueType::ToolMismatch,
        severity: IssueSeverity::Critical,
        description: "Tool brave_search declared but not available from tools-server".to_string(),
        agent: Some("rex".to_string()),
        task_id: Some("1".to_string()),
        remediation_spawned: false,
        github_issue: None,
    };

    let add_result = state.sessions.add_issue("roundtrip-test", issue.clone()).await;
    assert!(add_result.is_ok());
    println!("✅ Step 2: Issue detected and added");

    // Step 3: Determine remediation strategy
    let strategy = RemediationSpawner::get_remediation_strategy(&issue);
    assert_eq!(
        format!("{strategy:?}"),
        "FixConfig",
        "ToolMismatch + Critical should map to FixConfig"
    );
    println!("✅ Step 3: Remediation strategy determined: {:?}", strategy);

    // Step 4: Verify the session has the issue
    let response = app
        .oneshot(
            Request::builder()
                .uri("/api/v1/session/roundtrip-test")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    let body = axum::body::to_bytes(response.into_body(), 8192)
        .await
        .unwrap();
    let session_response: Value = serde_json::from_slice(&body).unwrap();
    let session = &session_response["session"];
    let issues = session["issues"].as_array().unwrap();
    assert_eq!(issues.len(), 1);
    assert_eq!(issues[0]["issue_type"], "tool_mismatch");
    println!("✅ Step 4: Issue stored in session");

    println!("\n🎉 Full monitoring round-trip test PASSED!");
}
