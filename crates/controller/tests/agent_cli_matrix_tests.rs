//! Agent × CLI Matrix Tests
//!
//! Comprehensive test suite that validates every agent can work with every supported CLI.
//! Tests validate:
//! 1. Container scripts render correctly
//! 2. Memory/system prompts contain expected content
//! 3. Config files are valid and contain required fields
//!
//! Run with:
//! ```sh
//! AGENT_TEMPLATES_PATH="$(pwd)/templates" cargo test -p controller --test agent_cli_matrix_tests
//! ```

#![allow(dead_code)] // Test fixtures have some unused fields for documentation
#![allow(clippy::disallowed_macros)] // println! is appropriate in tests
#![allow(clippy::needless_raw_string_hashes)] // Raw strings in test data are fine
#![allow(clippy::too_many_lines)] // Test data functions are naturally long
#![allow(clippy::doc_markdown)] // Commands in doc comments don't need backticks
#![allow(clippy::match_same_arms)] // Explicit arms improve test clarity
#![allow(clippy::struct_excessive_bools)] // TestResult struct needs multiple status flags

use controller::cli::types::CLIType;
use controller::crds::coderun::CLIConfig;
use controller::crds::{CodeRun, CodeRunSpec};
use controller::tasks::code::templates::CodeTemplateGenerator;
use controller::tasks::config::ControllerConfig;
use k8s_openapi::apimachinery::pkg::apis::meta::v1::ObjectMeta;
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

// ============================================================================
// Realistic Scenarios - Role-Based Test Data
// ============================================================================

/// A realistic task scenario for an agent based on their specialty
#[derive(Debug, Clone)]
struct TaskScenario {
    title: &'static str,
    repository: &'static str,
    working_directory: &'static str,
    task_prompt: &'static str,
}

// ============================================================================
// Test Matrix Definition
// ============================================================================

/// Agent definition for testing with a realistic scenario
#[derive(Debug, Clone)]
struct AgentDef {
    name: &'static str,
    github_app: &'static str,
    specialty: &'static str,
    primary_job: &'static str,
    supported_clis: Vec<CLIType>,
    keywords: Vec<&'static str>,
    scenario: TaskScenario,
}

/// Get the full agent matrix with realistic scenarios
fn get_agent_matrix() -> Vec<AgentDef> {
    vec![
        // ═══════════════════════════════════════════════════════════════════
        // CODING SPECIALISTS (all CLIs)
        // ═══════════════════════════════════════════════════════════════════
        AgentDef {
            name: "rex",
            github_app: "5DLabs-Rex",
            specialty: "Rust",
            primary_job: "coder",
            supported_clis: all_clis(),
            keywords: vec!["rust", "cargo", "Rust", "Cargo"],
            scenario: TaskScenario {
                title: "Add async retry logic to HTTP client",
                repository: "5dlabs/cto",
                working_directory: "crates/controller",
                task_prompt: r#"## Task: Implement Retry Logic for HTTP Client

Add exponential backoff retry logic to `src/http/client.rs`.

### Requirements
- Retry on connection errors and 429/503 status codes
- Exponential backoff: 100ms, 200ms, 400ms, 800ms, max 5 retries
- Use tokio for async delays
- Add jitter to prevent thundering herd
- Log retry attempts with tracing

### Acceptance Criteria
- [ ] New `RetryConfig` struct with configurable parameters
- [ ] `with_retry()` wrapper function for reqwest requests  
- [ ] Unit tests for retry logic
- [ ] Integration test with mock server"#,
            },
        },
        AgentDef {
            name: "blaze",
            github_app: "5DLabs-Blaze",
            specialty: "Frontend/React",
            primary_job: "coder",
            supported_clis: all_clis(),
            keywords: vec!["frontend", "react", "React", "UI", "component"],
            scenario: TaskScenario {
                title: "Create responsive data table component",
                repository: "5dlabs/console",
                working_directory: "src/components",
                task_prompt: r#"## Task: Create DataTable Component

Build a responsive data table for `src/components/DataTable/`.

### Requirements
- Column sorting (click headers)
- Text search filtering
- Pagination with configurable page size
- Responsive: stack on mobile, scroll on tablet, full on desktop
- Keyboard navigation support
- Loading and empty states

### Tech Stack
- React 18 with TypeScript
- TailwindCSS for styling
- React Table v8 for table logic

### Acceptance Criteria
- [ ] DataTable component with TypeScript generics
- [ ] Storybook stories for all states
- [ ] Unit tests with React Testing Library
- [ ] Responsive breakpoints working"#,
            },
        },
        AgentDef {
            name: "grizz",
            github_app: "5DLabs-Grizz",
            specialty: "Go",
            primary_job: "coder",
            supported_clis: all_clis(),
            keywords: vec!["go", "Go", "golang", "Golang"],
            scenario: TaskScenario {
                title: "Implement rate limiter middleware",
                repository: "5dlabs/gateway",
                working_directory: "internal/middleware",
                task_prompt: r#"## Task: Rate Limiter Middleware

Implement rate limiting in `internal/middleware/ratelimit.go`.

### Requirements
- Token bucket algorithm
- Per-client rate limiting (by API key or IP)
- Redis backend for distributed rate limit state
- Configurable limits per route
- Return proper 429 responses with Retry-After header

### Tech Stack
- Go 1.21+
- go-redis/redis for Redis client
- chi router middleware interface

### Acceptance Criteria
- [ ] RateLimiter struct with token bucket implementation
- [ ] Redis storage adapter
- [ ] Middleware function for chi router
- [ ] Unit tests with mock Redis
- [ ] Benchmark tests for performance"#,
            },
        },
        AgentDef {
            name: "nova",
            github_app: "5DLabs-Nova",
            specialty: "Node.js",
            primary_job: "coder",
            supported_clis: all_clis(),
            keywords: vec!["node", "Node", "TypeScript", "JavaScript"],
            scenario: TaskScenario {
                title: "Add WebSocket real-time notifications",
                repository: "5dlabs/api",
                working_directory: "src/notifications",
                task_prompt: r#"## Task: WebSocket Notifications

Add real-time notifications via WebSocket in `src/notifications/`.

### Requirements
- NestJS WebSocket gateway with Socket.io
- Redis adapter for multi-instance support
- JWT authentication for WebSocket connections
- Typed events with TypeScript
- Reconnection handling on client disconnect

### Tech Stack
- NestJS with @nestjs/websockets
- Socket.io with Redis adapter
- TypeScript strict mode

### Acceptance Criteria
- [ ] NotificationsGateway with handleConnection/handleDisconnect
- [ ] Redis pub/sub adapter configuration
- [ ] Authentication guard for WebSocket
- [ ] E2E tests for WebSocket connections
- [ ] Client SDK types exported"#,
            },
        },
        AgentDef {
            name: "tap",
            github_app: "5DLabs-Tap",
            specialty: "Expo Mobile",
            primary_job: "coder",
            supported_clis: all_clis(),
            keywords: vec!["mobile", "Expo", "iOS", "Android", "React Native"],
            scenario: TaskScenario {
                title: "Implement biometric authentication",
                repository: "5dlabs/mobile",
                working_directory: "src/features/auth",
                task_prompt: r#"## Task: Biometric Authentication

Add biometric auth in `src/features/auth/biometric/`.

### Requirements
- Face ID / Touch ID support via expo-local-authentication
- Secure storage of auth tokens with expo-secure-store
- Fallback to PIN/password if biometrics unavailable
- Settings screen to enable/disable biometric login
- Works on both iOS and Android

### Tech Stack
- Expo SDK 50
- expo-local-authentication
- expo-secure-store
- React Native with TypeScript

### Acceptance Criteria
- [ ] useBiometricAuth hook
- [ ] BiometricPrompt component
- [ ] Secure token storage utilities
- [ ] Platform-specific handling (iOS/Android)
- [ ] Unit tests with mocked native modules"#,
            },
        },
        AgentDef {
            name: "spark",
            github_app: "5DLabs-Spark",
            specialty: "Electron Desktop",
            primary_job: "coder",
            supported_clis: all_clis(),
            keywords: vec!["Electron", "desktop", "IPC", "tray"],
            scenario: TaskScenario {
                title: "Add system tray with quick actions",
                repository: "5dlabs/desktop",
                working_directory: "src/main",
                task_prompt: r#"## Task: System Tray Integration

Add system tray support in `src/main/tray.ts`.

### Requirements
- System tray icon with dynamic status indicator
- Context menu with: Start/Stop, Settings, About, Quit
- Click to show/hide main window
- Native notifications for status changes
- Platform-specific icons (macOS template, Windows ICO)

### Tech Stack
- Electron 28
- TypeScript
- electron-builder for packaging

### Acceptance Criteria
- [ ] TrayManager class with lifecycle management
- [ ] Dynamic icon updates based on app state
- [ ] Cross-platform context menu
- [ ] IPC handlers for renderer communication
- [ ] Proper cleanup on app quit"#,
            },
        },
        // ═══════════════════════════════════════════════════════════════════
        // SPECIALIZED AGENTS (limited CLI support)
        // ═══════════════════════════════════════════════════════════════════
        AgentDef {
            name: "bolt",
            github_app: "5DLabs-Bolt",
            specialty: "Deployment",
            primary_job: "deploy",
            supported_clis: vec![CLIType::Claude, CLIType::Factory],
            keywords: vec!["deploy", "CI", "CD", "pipeline"],
            scenario: TaskScenario {
                title: "Add blue-green deployment to Kubernetes",
                repository: "5dlabs/infra",
                working_directory: "k8s/deployments",
                task_prompt: r#"## Task: Blue-Green Deployment

Implement blue-green deployment in `k8s/deployments/`.

### Requirements
- Blue and green deployment manifests
- Service switching via label selectors
- Health check validation before switch
- Automated rollback on failure
- ArgoCD integration for GitOps

### Acceptance Criteria
- [ ] Blue/green deployment templates
- [ ] Switch script with health validation
- [ ] Rollback procedure documented
- [ ] ArgoCD Application manifest
- [ ] Runbook for manual intervention"#,
            },
        },
        AgentDef {
            name: "cipher",
            github_app: "5DLabs-Cipher",
            specialty: "Security",
            primary_job: "security",
            supported_clis: vec![CLIType::Claude, CLIType::Factory],
            keywords: vec!["security", "vulnerab", "audit"],
            scenario: TaskScenario {
                title: "Fix SQL injection vulnerability",
                repository: "5dlabs/api",
                working_directory: "src/users",
                task_prompt: r#"## Task: Fix SQL Injection (CVE-2024-XXXX)

Remediate SQL injection in `src/users/search.ts`.

### Vulnerability
The user search endpoint concatenates user input directly into SQL:
```
const query = `SELECT * FROM users WHERE name LIKE '%${search}%'`;
```

### Requirements
- Use parameterized queries with prepared statements
- Add input validation and sanitization
- Implement query builder pattern
- Add SQL injection test cases
- Update security documentation

### Acceptance Criteria
- [ ] Parameterized query implementation
- [ ] Input validation middleware
- [ ] Security test cases passing
- [ ] No raw SQL concatenation in codebase
- [ ] Security review checklist completed"#,
            },
        },
        AgentDef {
            name: "cleo",
            github_app: "5DLabs-Cleo",
            specialty: "Quality",
            primary_job: "quality",
            supported_clis: vec![CLIType::Claude, CLIType::Factory],
            keywords: vec!["quality", "review", "standard"],
            scenario: TaskScenario {
                title: "Enforce consistent error handling",
                repository: "5dlabs/cto",
                working_directory: "crates",
                task_prompt: r#"## Task: Standardize Error Handling

Implement consistent error handling across all crates.

### Current Issues
- Mix of anyhow::Error and custom error types
- Inconsistent error context/messages
- Missing error codes for API responses
- Logs don't include error chain

### Requirements
- Define error hierarchy with thiserror
- Consistent error context pattern
- Error codes enum for API
- Structured logging with tracing

### Acceptance Criteria
- [ ] Error types defined in each crate
- [ ] All functions use Result with proper types
- [ ] Error context added at boundaries
- [ ] API errors have stable codes
- [ ] Documentation for error handling"#,
            },
        },
        AgentDef {
            name: "tess",
            github_app: "5DLabs-Tess",
            specialty: "Testing",
            primary_job: "test",
            supported_clis: vec![CLIType::Claude, CLIType::Factory],
            keywords: vec!["test", "coverage", "spec"],
            scenario: TaskScenario {
                title: "Add integration test suite for API",
                repository: "5dlabs/api",
                working_directory: "tests/integration",
                task_prompt: r#"## Task: API Integration Tests

Build integration test suite in `tests/integration/`.

### Requirements
- Testcontainers for PostgreSQL and Redis
- Test fixtures with factory pattern
- API client wrapper for typed requests
- Coverage for all CRUD endpoints
- CI pipeline integration

### Test Categories
- Authentication flows
- User management
- Resource CRUD operations
- Error scenarios
- Rate limiting behavior

### Acceptance Criteria
- [ ] Test setup with testcontainers
- [ ] Factory functions for test data
- [ ] Tests for all API endpoints
- [ ] CI configuration for test run
- [ ] Coverage report generation"#,
            },
        },
        AgentDef {
            name: "stitch",
            github_app: "5DLabs-Stitch",
            specialty: "Code Review",
            primary_job: "review",
            supported_clis: vec![CLIType::Claude, CLIType::Factory],
            keywords: vec!["review", "PR", "feedback"],
            scenario: TaskScenario {
                title: "Review and improve PR #1234",
                repository: "5dlabs/api",
                working_directory: ".",
                task_prompt: r#"## Task: Review PR #1234 - Auth Refactor

Perform thorough code review of the authentication refactor.

### Review Checklist
- [ ] Security: No credentials in code, proper token handling
- [ ] Performance: No N+1 queries, efficient caching
- [ ] Testing: Adequate coverage, edge cases handled
- [ ] Documentation: API changes documented
- [ ] Breaking changes: Migration path clear

### Focus Areas
- JWT token validation logic
- Session management changes
- Database schema migrations
- API contract changes

### Deliverables
- Line-by-line review comments
- Security findings (if any)
- Suggested improvements
- Approval or request changes"#,
            },
        },
        AgentDef {
            name: "morgan",
            github_app: "5DLabs-Morgan",
            specialty: "Documentation",
            primary_job: "docs",
            supported_clis: vec![CLIType::Claude],
            keywords: vec!["document", "PRD", "spec"],
            scenario: TaskScenario {
                title: "Create API documentation from OpenAPI spec",
                repository: "5dlabs/docs",
                working_directory: "api",
                task_prompt: r#"## Task: API Documentation

Create developer documentation in `api/`.

### Requirements
- Parse OpenAPI 3.0 spec
- Generate endpoint documentation
- Add request/response examples
- Document error codes and handling
- Create quickstart guide
- Add authentication section

### Output Files
- `getting-started.md`
- `authentication.md`
- `endpoints/` directory with per-resource docs
- `errors.md`
- `examples/` with code samples

### Acceptance Criteria
- [ ] All endpoints documented
- [ ] Working code examples
- [ ] Error codes table
- [ ] Authentication guide
- [ ] Links validated"#,
            },
        },
        AgentDef {
            name: "atlas",
            github_app: "5DLabs-Atlas",
            specialty: "Integration",
            primary_job: "integration",
            supported_clis: vec![CLIType::Claude],
            keywords: vec!["merge", "conflict", "integrate"],
            scenario: TaskScenario {
                title: "Resolve merge conflicts in feature branch",
                repository: "5dlabs/cto",
                working_directory: ".",
                task_prompt: r#"## Task: Merge Feature Branch

Integrate `feature/new-scheduler` into `main`.

### Conflict Areas
- `src/scheduler/mod.rs` - Both branches modified
- `Cargo.toml` - Dependency version conflicts
- `tests/scheduler_tests.rs` - New tests on both sides

### Requirements
- Preserve all functionality from both branches
- Resolve Cargo.toml to latest compatible versions
- Merge test files keeping all test cases
- Ensure CI passes after merge
- No loss of commit history

### Acceptance Criteria
- [ ] All conflicts resolved
- [ ] Both feature sets working
- [ ] Tests passing
- [ ] Clean git history
- [ ] No regression in functionality"#,
            },
        },
    ]
}

fn all_clis() -> Vec<CLIType> {
    vec![
        CLIType::Claude,
        CLIType::Codex,
        CLIType::Cursor,
        CLIType::Factory,
        CLIType::Gemini,
        CLIType::OpenCode,
    ]
}

// ============================================================================
// Test Fixtures
// ============================================================================

/// Create a CodeRun with realistic scenario data from an agent definition
fn create_code_run_with_scenario(agent: &AgentDef, cli_type: CLIType) -> CodeRun {
    let mut settings = HashMap::new();
    settings.insert("approvalPolicy".to_string(), serde_json::json!("never"));
    settings.insert(
        "sandboxMode".to_string(),
        serde_json::json!("workspace-write"),
    );

    // Parse repository into org/repo format for URL
    let repo_url = format!("https://github.com/{}", agent.scenario.repository);

    CodeRun {
        metadata: ObjectMeta {
            name: Some(format!("{}-{}-run", agent.name, get_cli_name(cli_type))),
            namespace: Some("default".to_string()),
            ..Default::default()
        },
        spec: CodeRunSpec {
            run_type: agent.primary_job.to_string(),
            cli_config: Some(CLIConfig {
                cli_type,
                model: get_model_for_cli(cli_type),
                settings,
                max_tokens: Some(16000),
                temperature: Some(0.7),
                model_rotation: None,
            }),
            task_id: Some(42),
            service: extract_service_name(agent.scenario.repository),
            repository_url: repo_url.clone(),
            docs_repository_url: format!("{repo_url}-docs"),
            docs_project_directory: Some("docs".to_string()),
            working_directory: Some(agent.scenario.working_directory.to_string()),
            model: get_model_for_cli(cli_type),
            github_user: Some("test-user".to_string()),
            github_app: Some(agent.github_app.to_string()),
            context_version: 1,
            continue_session: false,
            overwrite_memory: false,
            docs_branch: "main".to_string(),
            env: HashMap::new(),
            env_from_secrets: Vec::new(),
            enable_docker: false,
            task_requirements: Some(agent.scenario.task_prompt.to_string()),
            service_account_name: None,
            linear_integration: None,
        },
        status: None,
    }
}

/// Legacy function for backwards compatibility
fn create_test_code_run(github_app: &str, cli_type: CLIType) -> CodeRun {
    let mut settings = HashMap::new();
    settings.insert("approvalPolicy".to_string(), serde_json::json!("never"));
    settings.insert(
        "sandboxMode".to_string(),
        serde_json::json!("workspace-write"),
    );

    CodeRun {
        metadata: ObjectMeta {
            name: Some("matrix-test-run".to_string()),
            namespace: Some("default".to_string()),
            ..Default::default()
        },
        spec: CodeRunSpec {
            run_type: "implementation".to_string(),
            cli_config: Some(CLIConfig {
                cli_type,
                model: get_model_for_cli(cli_type),
                settings,
                max_tokens: Some(16000),
                temperature: Some(0.7),
                model_rotation: None,
            }),
            task_id: Some(1),
            service: "matrix-test".to_string(),
            repository_url: "https://github.com/test/repo".to_string(),
            docs_repository_url: "https://github.com/test/docs".to_string(),
            docs_project_directory: Some("docs".to_string()),
            working_directory: Some("src".to_string()),
            model: get_model_for_cli(cli_type),
            github_user: Some("test-user".to_string()),
            github_app: Some(github_app.to_string()),
            context_version: 1,
            continue_session: false,
            overwrite_memory: false,
            docs_branch: "main".to_string(),
            env: HashMap::new(),
            env_from_secrets: Vec::new(),
            enable_docker: false,
            task_requirements: None,
            service_account_name: None,
            linear_integration: None,
        },
        status: None,
    }
}

/// Get a realistic model for each CLI type
fn get_model_for_cli(cli_type: CLIType) -> String {
    match cli_type {
        CLIType::Claude => "claude-sonnet-4-20250514".to_string(),
        CLIType::Codex => "o4-mini".to_string(),
        CLIType::Cursor => "claude-sonnet-4-20250514".to_string(),
        CLIType::Factory => "claude-sonnet-4-20250514".to_string(),
        CLIType::Gemini => "gemini-2.5-pro".to_string(),
        CLIType::OpenCode => "claude-sonnet-4-20250514".to_string(),
        _ => "test-model".to_string(),
    }
}

/// Extract service name from repository path
fn extract_service_name(repo: &str) -> String {
    repo.split('/').next_back().unwrap_or("unknown").to_string()
}

fn get_cli_name(cli_type: CLIType) -> &'static str {
    match cli_type {
        CLIType::Claude => "claude",
        CLIType::Codex => "codex",
        CLIType::Cursor => "cursor",
        CLIType::Factory => "factory",
        CLIType::Gemini => "gemini",
        CLIType::OpenCode => "opencode",
        _ => "unknown",
    }
}

fn get_cli_markers(cli_type: CLIType) -> Vec<&'static str> {
    match cli_type {
        CLIType::Claude => vec!["claude", "CLAUDE"],
        CLIType::Codex => vec!["codex", "CODEX"],
        CLIType::Cursor => vec!["cursor", "CURSOR"],
        CLIType::Factory => vec!["factory", "FACTORY"],
        CLIType::Gemini => vec!["gemini", "GEMINI"],
        CLIType::OpenCode => vec!["opencode", "OPENCODE"],
        _ => vec![],
    }
}

fn get_memory_file(cli_type: CLIType) -> &'static str {
    match cli_type {
        CLIType::Claude => "CLAUDE.md",
        CLIType::Gemini => "GEMINI.md",
        _ => "AGENTS.md",
    }
}

fn get_config_file(cli_type: CLIType) -> &'static str {
    match cli_type {
        CLIType::Claude => "settings.json",
        CLIType::Codex => "codex-config.toml",
        CLIType::Cursor => "cursor-cli-config.json",
        CLIType::Factory => "factory-cli-config.json",
        CLIType::Gemini => "settings.json",
        CLIType::OpenCode => "opencode-config.json",
        _ => "config.json",
    }
}

// ============================================================================
// Acceptance Criteria Validation
// ============================================================================

#[derive(Debug)]
struct TestResult {
    agent: String,
    cli: String,
    container_ok: bool,
    container_size: usize,
    container_has_cli_marker: bool,
    container_has_bash: bool,
    memory_ok: bool,
    memory_size: usize,
    memory_has_content: bool,
    config_ok: bool,
    config_size: usize,
    config_valid_format: bool,
    errors: Vec<String>,
}

impl TestResult {
    fn passed(&self) -> bool {
        self.container_ok
            && self.container_has_cli_marker
            && self.container_has_bash
            && self.memory_ok
            && self.memory_has_content
            && self.config_ok
            && self.config_valid_format
            && self.errors.is_empty()
    }
}

fn validate_container(content: &str, cli_type: CLIType) -> (bool, bool, bool, Vec<String>) {
    let mut errors = Vec::new();

    // Check for bash script markers
    let has_bash =
        content.contains("#!/bin/bash") || content.contains("set -e") || content.contains("set -");
    if !has_bash {
        errors.push("Container missing bash shebang or set -e".to_string());
    }

    // Check for CLI markers
    let markers = get_cli_markers(cli_type);
    let has_cli_marker = markers.iter().any(|m| content.contains(m));
    if !has_cli_marker {
        errors.push(format!(
            "Container missing CLI marker for {cli_type:?} (expected one of {markers:?})"
        ));
    }

    // Check minimum size (should be at least 5KB for full container)
    let size_ok = content.len() >= 5000;
    if !size_ok {
        let len = content.len();
        errors.push(format!(
            "Container too small: {len} bytes (expected >= 5000)"
        ));
    }

    (has_bash, has_cli_marker, size_ok, errors)
}

fn validate_memory(content: &str, agent: &AgentDef) -> (bool, Vec<String>) {
    let mut errors = Vec::new();

    // Check for markdown content
    let has_markdown = content.contains('#') || content.contains("- ") || content.contains("* ");
    if !has_markdown {
        errors.push("Memory missing markdown formatting".to_string());
    }

    // Check for agent identity markers
    let has_identity = content.contains("You are")
        || content.contains("Your role")
        || content.to_lowercase().contains("assistant");
    if !has_identity {
        errors.push("Memory missing agent identity".to_string());
    }

    // Check for specialty keywords (case-insensitive)
    let content_lower = content.to_lowercase();
    let has_specialty = agent
        .keywords
        .iter()
        .any(|kw| content_lower.contains(&kw.to_lowercase()));
    if !has_specialty {
        let name = agent.name;
        let keywords = &agent.keywords;
        errors.push(format!(
            "Memory missing specialty keywords for {name} (expected one of {keywords:?})"
        ));
    }

    (has_markdown && has_identity, errors)
}

fn validate_config(content: &str, cli_type: CLIType) -> (bool, Vec<String>) {
    let mut errors = Vec::new();

    let valid = match cli_type {
        CLIType::Codex => {
            // TOML validation
            content.contains('[') || content.contains('=')
        }
        _ => {
            // JSON validation
            serde_json::from_str::<serde_json::Value>(content).is_ok()
        }
    };

    if !valid {
        let format = if cli_type == CLIType::Codex {
            "TOML"
        } else {
            "JSON"
        };
        errors.push(format!("Config is not valid {format}"));
    }

    (valid, errors)
}

// ============================================================================
// Matrix Test Implementation
// ============================================================================

/// Check if templates are available for testing.
/// Returns true if AGENT_TEMPLATES_PATH is set and points to valid templates.
fn templates_available() -> bool {
    std::env::var("AGENT_TEMPLATES_PATH")
        .map(|p| PathBuf::from(p).join("_shared/container.sh.hbs").exists())
        .unwrap_or(false)
}

/// Skip test if templates aren't available
macro_rules! skip_if_no_templates {
    () => {
        if !templates_available() {
            eprintln!(
                "Skipping test: AGENT_TEMPLATES_PATH not set or templates not found.\n\
                 Run with: AGENT_TEMPLATES_PATH=\"$(pwd)/templates\" cargo test ..."
            );
            return;
        }
    };
}

fn run_matrix_test(agent: &AgentDef, cli_type: CLIType) -> TestResult {
    let config = ControllerConfig::default();
    let code_run = create_code_run_with_scenario(agent, cli_type);

    let mut result = TestResult {
        agent: agent.name.to_string(),
        cli: get_cli_name(cli_type).to_string(),
        container_ok: false,
        container_size: 0,
        container_has_cli_marker: false,
        container_has_bash: false,
        memory_ok: false,
        memory_size: 0,
        memory_has_content: false,
        config_ok: false,
        config_size: 0,
        config_valid_format: false,
        errors: Vec::new(),
    };

    // Generate templates
    let templates = match CodeTemplateGenerator::generate_all_templates(&code_run, &config) {
        Ok(t) => t,
        Err(e) => {
            result
                .errors
                .push(format!("Template generation failed: {e:?}"));
            return result;
        }
    };

    // Validate container.sh
    if let Some(container) = templates.get("container.sh") {
        result.container_size = container.len();
        let (has_bash, has_cli, _size_ok, mut errors) = validate_container(container, cli_type);
        result.container_has_bash = has_bash;
        result.container_has_cli_marker = has_cli;
        result.container_ok = !container.is_empty();
        result.errors.append(&mut errors);
    } else {
        result.errors.push("container.sh not generated".to_string());
    }

    // Validate memory file
    let memory_file = get_memory_file(cli_type);
    if let Some(memory) = templates.get(memory_file) {
        result.memory_size = memory.len();
        let (has_content, mut errors) = validate_memory(memory, agent);
        result.memory_has_content = has_content;
        result.memory_ok = !memory.is_empty();
        result.errors.append(&mut errors);
    } else {
        result.errors.push(format!("{memory_file} not generated"));
    }

    // Validate config file
    let config_file = get_config_file(cli_type);
    if let Some(config_content) = templates.get(config_file) {
        result.config_size = config_content.len();
        let (valid, mut errors) = validate_config(config_content, cli_type);
        result.config_valid_format = valid;
        result.config_ok = !config_content.is_empty();
        result.errors.append(&mut errors);
    } else {
        result.errors.push(format!("{config_file} not generated"));
    }

    result
}

// ============================================================================
// Tests
// ============================================================================

#[test]
fn test_full_matrix() {
    skip_if_no_templates!();
    let agents = get_agent_matrix();
    let mut results: Vec<TestResult> = Vec::new();
    let mut failed_tests: Vec<String> = Vec::new();

    println!("\n═══════════════════════════════════════════════════════════════");
    println!("║           Agent × CLI Matrix Test Results                   ║");
    println!("═══════════════════════════════════════════════════════════════\n");

    for agent in &agents {
        println!("Agent: {} ({})", agent.name, agent.specialty);
        println!("────────────────────────────────────────────────────────────────");

        for cli_type in &agent.supported_clis {
            let result = run_matrix_test(agent, *cli_type);
            let cli_name = get_cli_name(*cli_type);

            if result.passed() {
                println!(
                    "  ✓ {} | container: {}B | memory: {}B | config: {}B",
                    cli_name, result.container_size, result.memory_size, result.config_size
                );
            } else {
                println!("  ✗ {cli_name} | FAILED");
                for error in &result.errors {
                    println!("    - {error}");
                }
                failed_tests.push(format!("{} + {cli_name}", agent.name));
            }

            results.push(result);
        }
        println!();
    }

    // Summary
    let total = results.len();
    let passed = results.iter().filter(|r| r.passed()).count();
    let failed = total - passed;

    println!("═══════════════════════════════════════════════════════════════");
    println!("Summary: {passed}/{total} passed, {failed} failed");
    println!("═══════════════════════════════════════════════════════════════\n");

    if !failed_tests.is_empty() {
        println!("Failed tests:");
        for test in &failed_tests {
            println!("  - {test}");
        }
        panic!(
            "Matrix test failed: {failed} of {total} combinations failed\nFailed: {failed_tests:?}"
        );
    }
}

// Individual agent tests for faster iteration
#[test]
fn test_rex_all_clis() {
    skip_if_no_templates!();
    let agents = get_agent_matrix();
    let rex = agents.iter().find(|a| a.name == "rex").unwrap();

    for cli_type in &rex.supported_clis {
        let result = run_matrix_test(rex, *cli_type);
        assert!(
            result.passed(),
            "Rex + {:?} failed: {:?}",
            cli_type,
            result.errors
        );
    }
}

#[test]
fn test_specialized_agents() {
    skip_if_no_templates!();
    let agents = get_agent_matrix();
    let specialized = vec![
        "bolt", "cipher", "cleo", "tess", "stitch", "morgan", "atlas",
    ];

    for agent_name in specialized {
        let agent = agents
            .iter()
            .find(|a| a.name == agent_name)
            .unwrap_or_else(|| panic!("Agent {agent_name} not found"));

        for cli_type in &agent.supported_clis {
            let result = run_matrix_test(agent, *cli_type);
            assert!(
                result.passed(),
                "{} + {:?} failed: {:?}",
                agent_name,
                cli_type,
                result.errors
            );
        }
    }
}

#[test]
fn test_all_coding_agents_with_claude() {
    skip_if_no_templates!();
    let agents = get_agent_matrix();
    let coders = vec!["rex", "blaze", "grizz", "nova", "tap", "spark"];

    for agent_name in coders {
        let agent = agents.iter().find(|a| a.name == agent_name).unwrap();
        let result = run_matrix_test(agent, CLIType::Claude);
        assert!(
            result.passed(),
            "{} + Claude failed: {:?}",
            agent_name,
            result.errors
        );
    }
}

// ============================================================================
// Detailed Acceptance Criteria Tests
// ============================================================================

#[test]
fn test_container_has_required_sections() {
    skip_if_no_templates!();
    let config = ControllerConfig::default();
    let code_run = create_test_code_run("5DLabs-Rex", CLIType::Claude);

    let templates = CodeTemplateGenerator::generate_all_templates(&code_run, &config).unwrap();
    let container = templates.get("container.sh").unwrap();

    // Required sections
    assert!(
        container.contains("ENVIRONMENT") || container.contains("Environment"),
        "Container should have environment section"
    );
    assert!(
        container.contains("git") || container.contains("Git"),
        "Container should have git setup"
    );
    assert!(
        container.contains("task") || container.contains("Task"),
        "Container should have task handling"
    );
    assert!(
        container.contains("CLI") || container.contains("cli") || container.contains("claude"),
        "Container should have CLI invocation"
    );
}

#[test]
fn test_memory_files_have_agent_context() {
    skip_if_no_templates!();
    let agents = get_agent_matrix();
    let config = ControllerConfig::default();

    for agent in agents.iter().take(3) {
        // Test first 3 agents
        let code_run = create_test_code_run(agent.github_app, CLIType::Claude);
        let templates = CodeTemplateGenerator::generate_all_templates(&code_run, &config).unwrap();

        let memory = templates.get("CLAUDE.md").unwrap();

        // Should contain some context
        assert!(
            memory.len() > 100,
            "{} memory too short: {} bytes",
            agent.name,
            memory.len()
        );

        // Should have markdown structure
        assert!(
            memory.contains('#'),
            "{} memory should have markdown headers",
            agent.name
        );
    }
}

#[test]
fn test_config_files_are_valid_format() {
    skip_if_no_templates!();
    let config = ControllerConfig::default();

    let test_cases = vec![
        (CLIType::Claude, "settings.json", true),
        (CLIType::Codex, "codex-config.toml", false), // TOML, not JSON
        (CLIType::Cursor, "cursor-cli-config.json", true),
        (CLIType::Factory, "factory-cli-config.json", true),
        (CLIType::Gemini, "settings.json", true),
        (CLIType::OpenCode, "opencode-config.json", true),
    ];

    for (cli_type, config_file, is_json) in test_cases {
        let code_run = create_test_code_run("5DLabs-Rex", cli_type);
        let templates = CodeTemplateGenerator::generate_all_templates(&code_run, &config).unwrap();

        let config_content = templates
            .get(config_file)
            .unwrap_or_else(|| panic!("{config_file} not found for {cli_type:?}"));

        if is_json {
            assert!(
                serde_json::from_str::<serde_json::Value>(config_content).is_ok(),
                "{config_file} should be valid JSON for {cli_type:?}"
            );
        } else {
            // TOML validation
            assert!(
                config_content.contains('=') || config_content.contains('['),
                "{config_file} should have TOML syntax for {cli_type:?}"
            );
        }
    }
}

// ============================================================================
// Output Generation for Manual Inspection
// ============================================================================

/// Get the output directory for rendered templates
fn get_output_dir() -> PathBuf {
    let manifest_dir = env!("CARGO_MANIFEST_DIR");
    PathBuf::from(manifest_dir)
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .join("tests/agent-cli-matrix/output")
}

/// Write all rendered templates to the output directory for manual inspection.
///
/// Run with: AGENT_TEMPLATES_PATH="$(pwd)/templates" cargo test -p controller --test agent_cli_matrix_tests generate_output -- --nocapture --ignored
#[test]
#[ignore = "Run explicitly to generate output files for manual inspection"]
fn generate_output_for_inspection() {
    let output_dir = get_output_dir();
    let config = ControllerConfig::default();
    let agents = get_agent_matrix();

    println!("\n═══════════════════════════════════════════════════════════════");
    println!("║       Generating Templates for Manual Inspection            ║");
    println!("║       Using Realistic Role-Based Scenarios                  ║");
    println!("═══════════════════════════════════════════════════════════════");
    println!("\nOutput directory: {}\n", output_dir.display());

    // Create output directory if it doesn't exist
    fs::create_dir_all(&output_dir).expect("Failed to create output directory");

    for agent in &agents {
        for cli_type in &agent.supported_clis {
            let cli_name = get_cli_name(*cli_type);
            let code_run = create_code_run_with_scenario(agent, *cli_type);

            // Create agent/cli subdirectory
            let agent_cli_dir = output_dir.join(format!("{}-{}", agent.name, cli_name));
            fs::create_dir_all(&agent_cli_dir).expect("Failed to create agent-cli directory");

            // Write scenario info as README
            let readme_content = format!(
                r#"# {} + {} Test Output

## Agent Info
- **Name**: {} ({})
- **Specialty**: {}
- **GitHub App**: {}
- **CLI**: {:?}
- **Model**: {}

## Scenario
**Task**: {}

**Repository**: {}
**Working Directory**: {}

## Task Prompt
{}

---
*Generated by agent_cli_matrix_tests*
"#,
                agent.name.to_uppercase(),
                cli_name.to_uppercase(),
                agent.name,
                agent.github_app,
                agent.specialty,
                agent.github_app,
                cli_type,
                get_model_for_cli(*cli_type),
                agent.scenario.title,
                agent.scenario.repository,
                agent.scenario.working_directory,
                agent.scenario.task_prompt,
            );
            fs::write(agent_cli_dir.join("README.md"), readme_content)
                .expect("Failed to write README");

            // Generate templates
            match CodeTemplateGenerator::generate_all_templates(&code_run, &config) {
                Ok(templates) => {
                    println!(
                        "✓ {} + {} ({}) - {} files",
                        agent.name,
                        cli_name,
                        agent.scenario.title,
                        templates.len()
                    );

                    for (filename, content) in &templates {
                        let file_path = agent_cli_dir.join(filename);
                        fs::write(&file_path, content).expect("Failed to write template");
                        println!("  → {} ({} bytes)", filename, content.len());
                    }
                }
                Err(e) => {
                    println!("✗ {} + {}: FAILED - {:?}", agent.name, cli_name, e);
                }
            }
            println!();
        }
    }

    println!("═══════════════════════════════════════════════════════════════");
    println!("Output written to: {}", output_dir.display());
    println!("═══════════════════════════════════════════════════════════════\n");
}

/// Generate output for a single agent/CLI combination
///
/// Run with: AGENT_TEMPLATES_PATH="$(pwd)/templates" cargo test -p controller --test agent_cli_matrix_tests generate_single_output -- --nocapture --ignored
#[test]
#[ignore = "Run explicitly to generate single agent/CLI output for debugging"]
fn generate_single_output() {
    // Configure which agent/CLI to generate
    let agent_name = "rex";
    let cli_type = CLIType::Claude;

    let output_dir = get_output_dir();
    let config = ControllerConfig::default();
    let agents = get_agent_matrix();

    let agent = agents
        .iter()
        .find(|a| a.name == agent_name)
        .expect("Agent not found");

    let cli_name = get_cli_name(cli_type);
    let code_run = create_code_run_with_scenario(agent, cli_type);

    // Create output directory
    let agent_cli_dir = output_dir.join(format!("{}-{}", agent.name, cli_name));
    fs::create_dir_all(&agent_cli_dir).expect("Failed to create directory");

    println!("\n═══════════════════════════════════════════════════════════════");
    println!(
        "║  Generating Templates: {} + {}                         ",
        agent.name, cli_name
    );
    println!("═══════════════════════════════════════════════════════════════");
    println!("\nSCENARIO: {}", agent.scenario.title);
    println!("REPOSITORY: {}", agent.scenario.repository);
    println!("WORKING DIR: {}", agent.scenario.working_directory);
    println!("\n────────────────────────────────────────────────────────────────");
    println!("TASK PROMPT:");
    println!("────────────────────────────────────────────────────────────────");
    println!("{}", agent.scenario.task_prompt);
    println!("════════════════════════════════════════════════════════════════\n");

    match CodeTemplateGenerator::generate_all_templates(&code_run, &config) {
        Ok(templates) => {
            for (filename, content) in &templates {
                let file_path = agent_cli_dir.join(filename);
                fs::write(&file_path, content).expect("Failed to write template");

                println!("────────────────────────────────────────────────────────────────");
                println!("FILE: {} ({} bytes)", filename, content.len());
                println!("────────────────────────────────────────────────────────────────");

                // For smaller files, print the full content
                if content.len() < 5000 {
                    println!("{content}");
                } else {
                    // For larger files, print first and last parts
                    let lines: Vec<&str> = content.lines().collect();
                    println!("... First 50 lines ...\n");
                    for line in lines.iter().take(50) {
                        println!("{line}");
                    }
                    println!(
                        "\n... ({} lines total, see file for full content) ...",
                        lines.len()
                    );
                }
                println!();
            }
        }
        Err(e) => {
            println!("FAILED: {e:?}");
        }
    }

    println!("\nOutput written to: {}", agent_cli_dir.display());
}
