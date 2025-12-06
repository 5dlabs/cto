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
        };

        assert_eq!(config.num_tasks, 20);
        assert!(!config.expand);
        assert!(config.analyze);
        assert_eq!(config.complexity_threshold, 7);
        assert_eq!(config.model, Some("claude-sonnet".to_string()));
    }
}
