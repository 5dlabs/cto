//! Integration tests for the intake workflow.
//!
//! These tests verify that the intake command works correctly
//! with various configurations and input files.

use std::path::PathBuf;

use tasks::domain::docs::{
    generate_acceptance_criteria, generate_all_docs, generate_task_prompt, generate_task_xml,
};
use tasks::domain::routing::{infer_agent_hint, Agent};
use tasks::domain::IntakeConfig;
use tasks::entities::Task;
use tempfile::TempDir;

// Note: Arc, IntakeDomain, and FileStorage are used for future integration tests
// that require mocking AI providers.

/// Test that agent routing correctly identifies task types
mod routing_tests {
    use super::*;

    #[test]
    fn test_routing_frontend_tasks() {
        let test_cases = vec![
            (
                "Create React component",
                "Build a button component",
                Agent::Blaze,
            ),
            ("UI design system", "Implement CSS variables", Agent::Blaze),
            ("Next.js page", "Server-side rendering", Agent::Blaze),
            ("Vue component", "Create form component", Agent::Blaze),
        ];

        for (title, desc, expected) in test_cases {
            assert_eq!(
                infer_agent_hint(title, desc),
                expected,
                "Failed for: {title} - {desc}"
            );
        }
    }

    #[test]
    fn test_routing_backend_tasks() {
        let test_cases = vec![
            ("Rust API endpoint", "Create user handler", Agent::Rex),
            ("Backend service", "Database connection pool", Agent::Rex),
            ("Axum router", "Setup routing middleware", Agent::Rex),
            ("Cargo workspace", "Multi-crate setup", Agent::Rex),
        ];

        for (title, desc, expected) in test_cases {
            assert_eq!(
                infer_agent_hint(title, desc),
                expected,
                "Failed for: {title} - {desc}"
            );
        }
    }

    #[test]
    fn test_routing_mobile_tasks() {
        let test_cases = vec![
            ("Mobile app screen", "React Native navigation", Agent::Tap),
            ("iOS push notifications", "Background fetch", Agent::Tap),
            ("Expo config", "App store deployment", Agent::Tap),
        ];

        for (title, desc, expected) in test_cases {
            assert_eq!(
                infer_agent_hint(title, desc),
                expected,
                "Failed for: {title} - {desc}"
            );
        }
    }

    #[test]
    fn test_routing_devops_tasks() {
        let test_cases = vec![
            ("Kubernetes deployment", "Helm chart setup", Agent::Bolt),
            ("CI/CD pipeline", "GitHub Actions workflow", Agent::Bolt),
            ("Docker container", "Multi-stage build", Agent::Bolt),
            ("Terraform config", "AWS infrastructure", Agent::Bolt),
        ];

        for (title, desc, expected) in test_cases {
            assert_eq!(
                infer_agent_hint(title, desc),
                expected,
                "Failed for: {title} - {desc}"
            );
        }
    }

    // ===========================================================================
    // PR #72 Regression Tests - Auth/Security Implementation vs Audit
    // ===========================================================================
    // These tests ensure auth/security IMPLEMENTATION tasks go to implementation
    // agents, not Cipher. Cipher should ONLY get explicit security audits.

    #[test]
    fn test_auth_implementation_not_cipher() {
        // These are all IMPLEMENTATION tasks that should NOT go to Cipher
        let test_cases = vec![
            // JWT implementations
            (
                "JWT Authentication",
                "Implement JWT middleware for API",
                Agent::Rex,
            ),
            (
                "JWT Token Validation",
                "Validate JWT tokens in requests",
                Agent::Rex,
            ),
            // OAuth implementations
            (
                "OAuth2 Token Management",
                "Implement OAuth2 flow with Effect TypeScript",
                Agent::Nova,
            ),
            (
                "OAuth Provider Setup",
                "Configure OAuth providers for login",
                Agent::Rex,
            ),
            // RBAC implementations
            (
                "RBAC Middleware",
                "Role-based access control implementation",
                Agent::Rex,
            ),
            (
                "RBAC Implementation",
                "Implement role-based permissions",
                Agent::Rex,
            ),
            // Auth flows
            (
                "Authentication Flow",
                "Build login/logout functionality",
                Agent::Rex,
            ),
            (
                "User Authentication",
                "Implement user auth service",
                Agent::Rex,
            ),
            // Password handling
            (
                "Password Reset",
                "Implement password reset flow",
                Agent::Rex,
            ),
            (
                "Password Hashing",
                "Add bcrypt password hashing",
                Agent::Rex,
            ),
        ];

        for (title, desc, expected) in test_cases {
            let actual = infer_agent_hint(title, desc);
            assert_ne!(
                actual,
                Agent::Cipher,
                "Auth implementation '{}' should NOT go to Cipher (got {:?}, expected {:?})",
                title,
                actual,
                expected
            );
        }
    }

    #[test]
    fn test_security_audit_goes_to_cipher() {
        // These are AUDIT/REVIEW tasks that SHOULD go to Cipher
        let test_cases = vec![
            (
                "Security Audit",
                "Review codebase for vulnerabilities",
                Agent::Cipher,
            ),
            (
                "Security Review",
                "Audit authentication implementation",
                Agent::Cipher,
            ),
            (
                "Vulnerability Scan",
                "Check for security vulnerabilities",
                Agent::Cipher,
            ),
            (
                "Penetration Test",
                "Security testing of the API",
                Agent::Cipher,
            ),
            (
                "Security Analysis",
                "Analyze security posture",
                Agent::Cipher,
            ),
        ];

        for (title, desc, expected) in test_cases {
            assert_eq!(
                infer_agent_hint(title, desc),
                expected,
                "Security audit '{}' should go to Cipher",
                title
            );
        }
    }

    // ===========================================================================
    // PR #72 Regression Tests - Go/gRPC Tasks
    // ===========================================================================

    #[test]
    fn test_go_grpc_tasks() {
        let test_cases = vec![
            // Go tasks
            (
                "Admin API - Go Service",
                "Go/gRPC backend service",
                Agent::Grizz,
            ),
            ("gRPC Server", "Implement gRPC service", Agent::Grizz),
            (
                "Protobuf Definitions",
                "Define protobuf messages",
                Agent::Grizz,
            ),
            ("Go Middleware", "Chi router middleware", Agent::Grizz),
            // Even with auth keywords, Go tasks should go to Grizz
            (
                "JWT Authentication",
                "Go/gRPC backend with JWT",
                Agent::Grizz,
            ),
            ("RBAC Service", "Go gRPC RBAC implementation", Agent::Grizz),
        ];

        for (title, desc, expected) in test_cases {
            assert_eq!(
                infer_agent_hint(title, desc),
                expected,
                "Go/gRPC task '{}' should go to Grizz",
                title
            );
        }
    }

    // ===========================================================================
    // PR #72 Regression Tests - Node/Bun/Effect Tasks
    // ===========================================================================

    #[test]
    fn test_nova_tasks() {
        let test_cases = vec![
            // Bun/Elysia tasks
            (
                "Integration Service",
                "Bun with Elysia framework",
                Agent::Nova,
            ),
            (
                "Slack Delivery Service",
                "Bun Elysia webhook integration",
                Agent::Nova,
            ),
            (
                "Effect Schema",
                "Effect TypeScript schema definitions",
                Agent::Nova,
            ),
            // Even with auth, Node tasks should go to Nova
            (
                "OAuth2 Flow",
                "Effect TypeScript OAuth2 implementation",
                Agent::Nova,
            ),
            ("Webhook Service", "Node.js webhook handler", Agent::Nova),
            (
                "Drizzle Models",
                "Drizzle ORM schema for integrations",
                Agent::Nova,
            ),
        ];

        for (title, desc, expected) in test_cases {
            assert_eq!(
                infer_agent_hint(title, desc),
                expected,
                "Nova task '{}' should go to Nova, not {:?}",
                title,
                infer_agent_hint(title, desc)
            );
        }
    }

    // ===========================================================================
    // PR #72 Regression Tests - Backend Tasks NOT to Tap
    // ===========================================================================

    #[test]
    fn test_backend_tasks_not_tap() {
        // These backend tasks were incorrectly assigned to Tap in PR #72
        let test_cases = vec![
            (
                "Rate Limiting Service",
                "Implement API rate limiting",
                Agent::Rex,
            ),
            ("Prometheus Metrics", "Add metrics endpoints", Agent::Rex),
            (
                "Message Queue Worker",
                "Process background jobs",
                Agent::Rex,
            ),
            ("Cache Service", "Redis caching layer", Agent::Rex),
        ];

        for (title, desc, expected) in test_cases {
            let actual = infer_agent_hint(title, desc);
            assert_ne!(
                actual,
                Agent::Tap,
                "Backend task '{}' should NOT go to Tap (mobile)",
                title
            );
            assert_eq!(
                actual, expected,
                "Backend task '{}' should go to {:?}",
                title, expected
            );
        }
    }

    // ===========================================================================
    // PR #72 Regression Tests - Dashboard/UI Tasks
    // ===========================================================================

    #[test]
    fn test_dashboard_ui_tasks() {
        // Dashboard and UI tasks should go to Blaze, even if they have
        // backend-sounding context
        let test_cases = vec![
            ("Admin Dashboard", "Build admin dashboard UI", Agent::Blaze),
            (
                "Notifications Page",
                "React notifications list page",
                Agent::Blaze,
            ),
            ("Settings Page", "User settings UI component", Agent::Blaze),
            (
                "Analytics Dashboard",
                "Chart.js analytics visualization",
                Agent::Blaze,
            ),
            (
                "Real-time Feed",
                "WebSocket notification feed component",
                Agent::Blaze,
            ),
            // Auth UI should go to Blaze
            (
                "Authentication Layout",
                "Login/signup UI components",
                Agent::Blaze,
            ),
            ("OAuth Login Page", "Social login buttons UI", Agent::Blaze),
        ];

        for (title, desc, expected) in test_cases {
            assert_eq!(
                infer_agent_hint(title, desc),
                expected,
                "UI task '{}' should go to Blaze",
                title
            );
        }
    }

    // ===========================================================================
    // Explicit Agent Name Tests
    // ===========================================================================

    #[test]
    fn test_explicit_agent_name_in_title() {
        // Explicit agent names in parentheses should override keyword detection
        let test_cases = vec![
            (
                "Integration Service (Nova - Bun/Elysia)",
                "OAuth2 implementation",
                Agent::Nova,
            ),
            (
                "Admin API (Grizz - Go/gRPC)",
                "JWT authentication",
                Agent::Grizz,
            ),
            (
                "Notification Router (Rex - Rust/Axum)",
                "Rate limiting",
                Agent::Rex,
            ),
            ("Web Console (Blaze - React)", "Auth flow UI", Agent::Blaze),
        ];

        for (title, desc, expected) in test_cases {
            assert_eq!(
                infer_agent_hint(title, desc),
                expected,
                "Explicit agent in '{}' should be respected",
                title
            );
        }
    }
}

/// Test documentation generation
mod docs_tests {
    use super::*;

    fn sample_task() -> Task {
        let mut task = Task::new("1", "Implement User API", "Create CRUD endpoints for users");
        task.details = "Use Axum framework with PostgreSQL".to_string();
        task.test_strategy = "Unit tests for handlers".to_string();
        task.agent_hint = Some("rex".to_string());
        task
    }

    #[test]
    fn test_xml_generation_structure() {
        let task = sample_task();
        let xml = generate_task_xml(&task);

        // Check XML structure
        assert!(xml.starts_with(r#"<?xml version="1.0" encoding="UTF-8"?>"#));
        assert!(xml.contains(r#"<task id="1""#));
        assert!(xml.contains("<meta>"));
        assert!(xml.contains("</meta>"));
        assert!(xml.contains("<role>"));
        assert!(xml.contains("<context>"));
        assert!(xml.contains("<requirements>"));
        assert!(xml.contains("<acceptance_criteria>"));
        assert!(xml.contains("<validation>"));
        assert!(xml.contains("<deliverables>"));
        assert!(xml.contains("</task>"));
    }

    #[test]
    fn test_xml_contains_task_content() {
        let task = sample_task();
        let xml = generate_task_xml(&task);

        assert!(xml.contains("Implement User API"));
        assert!(xml.contains("Create CRUD endpoints"));
        assert!(xml.contains("Axum framework"));
        assert!(xml.contains("rex"));
        assert!(xml.contains("Rust Engineer"));
    }

    #[test]
    fn test_prompt_markdown_structure() {
        let task = sample_task();
        let md = generate_task_prompt(&task);

        assert!(md.contains("# Task 1: Implement User API"));
        assert!(md.contains("## Role"));
        assert!(md.contains("## Goal"));
        assert!(md.contains("## Requirements"));
        assert!(md.contains("## Acceptance Criteria"));
        assert!(md.contains("## Constraints"));
    }

    #[test]
    fn test_acceptance_criteria_markdown() {
        let task = sample_task();
        let md = generate_acceptance_criteria(&task);

        assert!(md.contains("# Acceptance Criteria: Task 1"));
        assert!(md.contains("- [ ]"));
        assert!(md.contains("All requirements implemented"));
        assert!(md.contains("Tests passing"));
        assert!(md.contains("PR created"));
    }

    #[tokio::test]
    async fn test_generate_all_docs_creates_files() {
        let temp_dir = TempDir::new().unwrap();
        let output_dir = temp_dir.path().to_path_buf();

        let tasks = vec![
            {
                let mut t = Task::new("1", "Task One", "Description one");
                t.agent_hint = Some("rex".to_string());
                t
            },
            {
                let mut t = Task::new("2", "Task Two", "Description two");
                t.agent_hint = Some("blaze".to_string());
                t
            },
        ];

        let result = generate_all_docs(&tasks, &output_dir).await.unwrap();

        assert_eq!(result.task_dirs_created, 2);
        assert_eq!(result.xml_files, 2);
        assert_eq!(result.prompt_files, 2);
        assert_eq!(result.acceptance_files, 2);

        // Check files exist
        assert!(output_dir.join("task-1/prompt.xml").exists());
        assert!(output_dir.join("task-1/prompt.md").exists());
        assert!(output_dir.join("task-1/acceptance.md").exists());
        assert!(output_dir.join("task-2/prompt.xml").exists());
        assert!(output_dir.join("task-2/prompt.md").exists());
        assert!(output_dir.join("task-2/acceptance.md").exists());
    }
}

/// Test intake configuration
mod config_tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = IntakeConfig::default();

        assert_eq!(config.num_tasks, 15);
        assert!(config.expand);
        assert!(config.analyze);
        assert!(config.research);
        assert_eq!(config.complexity_threshold, 5);
        assert_eq!(config.prd_path, PathBuf::from(".tasks/docs/prd.txt"));
        assert_eq!(config.output_dir, PathBuf::from(".tasks"));
    }

    #[test]
    fn test_config_with_custom_values() {
        let config = IntakeConfig {
            prd_path: PathBuf::from("custom/prd.txt"),
            architecture_path: Some(PathBuf::from("custom/arch.md")),
            num_tasks: 20,
            expand: false,
            analyze: true,
            complexity_threshold: 7,
            research: false,
            model: Some("claude-sonnet".to_string()),
            output_dir: PathBuf::from("custom/output"),
            repository: Some("5dlabs/test".to_string()),
            service: Some("test-service".to_string()),
            docs_repository: Some("5dlabs/test".to_string()),
            docs_project_directory: Some("test-service".to_string()),
        };

        assert_eq!(config.num_tasks, 20);
        assert!(!config.expand);
        assert!(config.analyze);
        assert_eq!(config.complexity_threshold, 7);
        assert_eq!(config.model, Some("claude-sonnet".to_string()));
    }
}
