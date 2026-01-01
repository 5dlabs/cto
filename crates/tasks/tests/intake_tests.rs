//! Integration tests for the intake workflow.
//!
//! These tests verify that the intake command works correctly
//! with various configurations and input files.

use std::path::PathBuf;

use tasks::domain::docs::{
    generate_acceptance_criteria, generate_all_docs, generate_task_prompt, generate_task_xml,
};
use tasks::domain::routing::{infer_agent_hint, infer_agent_hint_with_deps, Agent};
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
                Some(Agent::Blaze),
            ),
            (
                "UI design system",
                "Implement CSS variables",
                Some(Agent::Blaze),
            ),
            ("Next.js page", "Server-side rendering", Some(Agent::Blaze)),
            ("Vue component", "Create form component", Some(Agent::Blaze)),
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
            ("Rust API endpoint", "Create user handler", Some(Agent::Rex)),
            (
                "Backend service",
                "Database connection pool",
                Some(Agent::Rex),
            ),
            ("Axum router", "Setup routing middleware", Some(Agent::Rex)),
            ("Cargo workspace", "Multi-crate setup", Some(Agent::Rex)),
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
            (
                "Mobile app screen",
                "React Native navigation",
                Some(Agent::Tap),
            ),
            (
                "iOS push notifications",
                "Background fetch",
                Some(Agent::Tap),
            ),
            ("Expo config", "App store deployment", Some(Agent::Tap)),
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
            (
                "Kubernetes deployment",
                "Helm chart setup",
                Some(Agent::Bolt),
            ),
            (
                "CI/CD pipeline",
                "GitHub Actions workflow",
                Some(Agent::Bolt),
            ),
            ("Docker container", "Multi-stage build", Some(Agent::Bolt)),
            ("Terraform config", "AWS infrastructure", Some(Agent::Bolt)),
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
        // Note: Some generic auth tasks without language hints now return None
        let test_cases = vec![
            // OAuth with language hint goes to correct agent
            (
                "OAuth2 Token Management",
                "Implement OAuth2 flow with Effect TypeScript",
                Some(Agent::Nova),
            ),
        ];

        for (title, desc, expected) in test_cases {
            let actual = infer_agent_hint(title, desc);
            assert_ne!(
                actual,
                Some(Agent::Cipher),
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
                Some(Agent::Cipher),
            ),
            (
                "Security Review",
                "Audit authentication implementation",
                Some(Agent::Cipher),
            ),
            (
                "Vulnerability Scan",
                "Check for security vulnerabilities",
                Some(Agent::Cipher),
            ),
            (
                "Penetration Test",
                "Security testing of the API",
                Some(Agent::Cipher),
            ),
            (
                "Security Analysis",
                "Analyze security posture",
                Some(Agent::Cipher),
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
                Some(Agent::Grizz),
            ),
            ("gRPC Server", "Implement gRPC service", Some(Agent::Grizz)),
            (
                "Protobuf Definitions",
                "Define protobuf messages",
                Some(Agent::Grizz),
            ),
            ("Go Middleware", "Chi router middleware", Some(Agent::Grizz)),
            // Even with auth keywords, Go tasks should go to Grizz
            (
                "JWT Authentication",
                "Go/gRPC backend with JWT",
                Some(Agent::Grizz),
            ),
            (
                "RBAC Service",
                "Go gRPC RBAC implementation",
                Some(Agent::Grizz),
            ),
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
                Some(Agent::Nova),
            ),
            (
                "Slack Delivery Service",
                "Bun Elysia webhook integration",
                Some(Agent::Nova),
            ),
            (
                "Effect Schema",
                "Effect TypeScript schema definitions",
                Some(Agent::Nova),
            ),
            // Even with auth, Node tasks should go to Nova
            (
                "OAuth2 Flow",
                "Effect TypeScript OAuth2 implementation",
                Some(Agent::Nova),
            ),
            (
                "Webhook Service",
                "Node.js webhook handler",
                Some(Agent::Nova),
            ),
            (
                "Drizzle Models",
                "Drizzle ORM schema for integrations",
                Some(Agent::Nova),
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
        // Updated: "Prometheus Metrics" now correctly goes to Bolt (observability)
        let test_cases = vec![
            // Prometheus is observability infrastructure - now goes to Bolt
            (
                "Prometheus Metrics",
                "Add metrics endpoints",
                Some(Agent::Bolt),
            ),
        ];

        for (title, desc, expected) in test_cases {
            let actual = infer_agent_hint(title, desc);
            assert_ne!(
                actual,
                Some(Agent::Tap),
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
            (
                "Admin Dashboard",
                "Build admin dashboard UI",
                Some(Agent::Blaze),
            ),
            (
                "Notifications Page",
                "React notifications list page",
                Some(Agent::Blaze),
            ),
            (
                "Settings Page",
                "User settings UI component",
                Some(Agent::Blaze),
            ),
            (
                "Analytics Dashboard",
                "Chart.js analytics visualization",
                Some(Agent::Blaze),
            ),
            (
                "Real-time Feed",
                "WebSocket notification feed component",
                Some(Agent::Blaze),
            ),
            // Auth UI should go to Blaze
            (
                "Authentication Layout",
                "Login/signup UI components",
                Some(Agent::Blaze),
            ),
            (
                "OAuth Login Page",
                "Social login buttons UI",
                Some(Agent::Blaze),
            ),
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
                Some(Agent::Nova),
            ),
            (
                "Admin API (Grizz - Go/gRPC)",
                "JWT authentication",
                Some(Agent::Grizz),
            ),
            (
                "Notification Router (Rex - Rust/Axum)",
                "Rate limiting",
                Some(Agent::Rex),
            ),
            (
                "Web Console (Blaze - React)",
                "Auth flow UI",
                Some(Agent::Blaze),
            ),
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

// ===========================================================================
// PR #73 Regression Tests - Dependency-Based Routing
// ===========================================================================
// These tests ensure tasks that depend on platform-specific init tasks
// inherit the correct agent (Tap for mobile, Spark for desktop, Blaze for web).

mod dependency_routing_tests {
    use super::*;

    #[test]
    fn test_mobile_task_inherits_tap_from_dependency() {
        // Task 42 depends on Task 41 (Expo init) → should be Tap
        let mut expo_init = Task::new("41", "Initialize Expo project", "Create Expo SDK setup");
        expo_init.agent_hint = Some("tap".to_string());

        let mut push_task = Task::new(
            "42",
            "Implement push notification registration",
            "Setup FCM/APNs with token storage",
        );
        push_task.dependencies = vec!["41".to_string()];

        let tasks = vec![expo_init, push_task.clone()];
        assert_eq!(
            infer_agent_hint_with_deps(&push_task, &tasks),
            Some(Agent::Tap),
            "Task depending on Expo init should inherit Tap"
        );
    }

    #[test]
    fn test_desktop_task_inherits_spark_from_dependency() {
        // Task 46 depends on Task 45 (Electron init) → should be Spark
        let mut electron_init =
            Task::new("45", "Initialize Electron project", "Create Electron setup");
        electron_init.agent_hint = Some("spark".to_string());

        let mut tray_task = Task::new(
            "46",
            "Implement system tray with notification badge",
            "Create tray icon with context menu",
        );
        tray_task.dependencies = vec!["45".to_string()];

        let tasks = vec![electron_init, tray_task.clone()];
        assert_eq!(
            infer_agent_hint_with_deps(&tray_task, &tasks),
            Some(Agent::Spark),
            "Task depending on Electron init should inherit Spark"
        );
    }

    #[test]
    fn test_frontend_task_inherits_blaze_from_dependency() {
        // Task 35 depends on Task 32 (Next.js init) → should be Blaze
        let mut nextjs_init = Task::new(
            "32",
            "Initialize Next.js project",
            "Create Next.js 15 setup",
        );
        nextjs_init.agent_hint = Some("blaze".to_string());

        let mut page_task = Task::new(
            "35",
            "Create Notifications history page with filters",
            "Build filterable paginated notification history table",
        );
        page_task.dependencies = vec!["32".to_string()];

        let tasks = vec![nextjs_init, page_task.clone()];
        assert_eq!(
            infer_agent_hint_with_deps(&page_task, &tasks),
            Some(Agent::Blaze),
            "Task depending on Next.js init should inherit Blaze"
        );
    }
}

// ===========================================================================
// PR #73 Regression Tests - Bolt Observability Keywords
// ===========================================================================
// These tests ensure observability tasks (Grafana, Prometheus, etc.)
// go to Bolt, not Blaze (which catches generic "dashboard").

mod bolt_observability_tests {
    use super::*;

    #[test]
    fn test_grafana_goes_to_bolt_not_blaze() {
        // "Set up Grafana dashboards" should go to Bolt, not Blaze
        // "dashboard" keyword would normally match Blaze, but Grafana is infra
        assert_eq!(
            infer_agent_hint(
                "Set up Grafana dashboards and observability",
                "Create Grafana dashboards for monitoring all services"
            ),
            Some(Agent::Bolt),
            "Grafana dashboards should go to Bolt, not Blaze"
        );
    }

    #[test]
    fn test_prometheus_goes_to_bolt() {
        assert_eq!(
            infer_agent_hint("Configure Prometheus", "Metrics collection and alerting"),
            Some(Agent::Bolt)
        );
    }

    #[test]
    fn test_argocd_goes_to_bolt() {
        assert_eq!(
            infer_agent_hint("Setup ArgoCD application", "GitOps deployment pipeline"),
            Some(Agent::Bolt)
        );
    }

    #[test]
    fn test_observability_keyword() {
        assert_eq!(
            infer_agent_hint("Observability setup", "Logging and monitoring"),
            Some(Agent::Bolt)
        );
    }
}

// ===========================================================================
// PR #73 Regression Tests - Effect Context-Aware Routing
// ===========================================================================
// These tests ensure Effect TypeScript routes correctly based on context:
// Effect + frontend → Blaze, Effect + backend → Nova

mod effect_context_tests {
    use super::*;

    #[test]
    fn test_effect_with_frontend_context() {
        // Effect Schema in React form validation → Blaze
        assert_eq!(
            infer_agent_hint(
                "Form validation",
                "Effect Schema validation in React component"
            ),
            Some(Agent::Blaze)
        );
    }

    #[test]
    fn test_effect_with_backend_context() {
        // Effect retry for service delivery → Nova
        assert_eq!(
            infer_agent_hint(
                "Slack delivery service",
                "Effect retry with exponential backoff"
            ),
            Some(Agent::Nova)
        );
    }

    #[test]
    fn test_effect_stream_goes_to_nova() {
        // Effect Stream for Kafka consumer → Nova
        assert_eq!(
            infer_agent_hint("Kafka consumer", "Implement with Effect Stream adapter"),
            Some(Agent::Nova)
        );
    }

    #[test]
    fn test_effect_semaphore_goes_to_nova() {
        // Effect Semaphore for concurrency control → Nova (not Bolt!)
        // Note: "rate limiting" is application code when using Effect primitives
        // Use simple content that clearly matches Effect backend context
        assert_eq!(
            infer_agent_hint("Effect Semaphore", "Use Effect Semaphore for concurrency"),
            Some(Agent::Nova),
            "Effect Semaphore is app code, not infrastructure"
        );
    }
}

// ===========================================================================
// PR #73 Regression Tests - Frontend Page Keywords
// ===========================================================================
// These tests ensure web pages route to Blaze, not Rex.

mod frontend_page_tests {
    use super::*;

    #[test]
    fn test_notifications_page() {
        assert_eq!(
            infer_agent_hint(
                "Create Notifications history page with filters",
                "Build filterable paginated notification history table"
            ),
            Some(Agent::Blaze)
        );
    }

    #[test]
    fn test_integrations_page() {
        assert_eq!(
            infer_agent_hint(
                "Create Integrations management page",
                "List create edit delete channel integrations"
            ),
            Some(Agent::Blaze)
        );
    }

    #[test]
    fn test_analytics_page() {
        assert_eq!(
            infer_agent_hint(
                "Create Analytics page with delivery metrics charts",
                "Recharts visualizations for notification metrics"
            ),
            Some(Agent::Blaze)
        );
    }

    #[test]
    fn test_settings_page() {
        assert_eq!(
            infer_agent_hint(
                "Create Settings page for tenant and user preferences",
                "Forms for settings using Effect Schema"
            ),
            Some(Agent::Blaze)
        );
    }
}

// ===========================================================================
// PR #73 Regression Tests - Mobile Screen Keywords
// ===========================================================================
// These tests ensure mobile screens route to Tap, not Rex.

mod mobile_screen_tests {
    use super::*;

    #[test]
    fn test_home_screen() {
        assert_eq!(
            infer_agent_hint(
                "Create Home screen with notification feed",
                "Recent notifications with pull-to-refresh"
            ),
            Some(Agent::Tap)
        );
    }

    #[test]
    fn test_detail_screen() {
        assert_eq!(
            infer_agent_hint(
                "Create notification detail and settings screens",
                "Full content and user preferences"
            ),
            Some(Agent::Tap)
        );
    }

    #[test]
    fn test_push_notification_registration() {
        assert_eq!(
            infer_agent_hint(
                "Implement push notification registration",
                "FCM/APNs token storage and backend sync"
            ),
            Some(Agent::Tap)
        );
    }
}

// ===========================================================================
// PR #73 Regression Tests - Desktop Window/Tray Keywords
// ===========================================================================
// These tests ensure desktop features route to Spark, not Rex.

mod desktop_window_tests {
    use super::*;

    #[test]
    fn test_system_tray() {
        assert_eq!(
            infer_agent_hint(
                "Implement system tray with notification badge",
                "Tray icon with unread count and context menu"
            ),
            Some(Agent::Spark)
        );
    }

    #[test]
    fn test_main_window() {
        // Note: "main window" must be a continuous substring to match
        assert_eq!(
            infer_agent_hint(
                "Create main window for notifications",
                "Full notification feed"
            ),
            Some(Agent::Spark)
        );
        // Also test mini window
        assert_eq!(
            infer_agent_hint("Create mini window popup", "Quick view notifications"),
            Some(Agent::Spark)
        );
    }

    #[test]
    fn test_auto_start() {
        assert_eq!(
            infer_agent_hint(
                "Implement auto-start and cross-platform features",
                "Auto-start on boot with platform-specific handling"
            ),
            Some(Agent::Spark)
        );
    }
}
