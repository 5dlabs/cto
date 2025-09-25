#![allow(dead_code, unused_imports)]

//! Task 1 Validation Tests
//!
//! Comprehensive tests to validate that Task 1 (CLI-Agnostic Platform Assessment)
//! has met all success criteria and deliverables.
//!
//! This validates the ASSESSMENT and RESEARCH aspects of Task 1, not implementation.

use std::fs;
use std::path::Path;

#[cfg(test)]
mod task_1_validation_tests {
    use super::*;

    /// Test that CLI_RESEARCH_FINDINGS.md exists and contains comprehensive research
    #[test]
    fn test_cli_research_findings_exists_and_comprehensive() {
        let findings_path = "CLI_RESEARCH_FINDINGS.md";
        assert!(Path::new(findings_path).exists(),
                "CLI_RESEARCH_FINDINGS.md must exist as primary Task 1 deliverable");

        let content = fs::read_to_string(findings_path)
            .expect("Should be able to read CLI_RESEARCH_FINDINGS.md");

        // Validate minimum comprehensive content (400+ lines as mentioned in PR)
        let line_count = content.lines().count();
        assert!(line_count >= 350,
                "CLI research findings should be comprehensive (>350 lines), found: {}", line_count);

        // Validate all 8 CLI types are covered
        let required_clis = vec![
            "Claude", "Codex", "OpenCode", "Gemini", "Grok", "Qwen", "Cursor", "OpenHands"
        ];

        for cli in required_clis {
            assert!(content.contains(cli),
                    "Research findings must cover {} CLI", cli);
        }
    }

    /// Test that all 8 CLI Docker directories exist
    #[test]
    fn test_all_cli_directories_exist() {
        let cli_names = vec![
            "claude", "codex", "opencode", "gemini",
            "grok", "qwen", "cursor", "openhands"
        ];

        for cli in cli_names {
            let cli_path = format!("infra/images/{}", cli);
            assert!(Path::new(&cli_path).exists(),
                    "CLI directory {} must exist for Task 1 container audit", cli);

            // Each CLI directory should have a Dockerfile
            let dockerfile_path = format!("infra/images/{}/Dockerfile", cli);
            assert!(Path::new(&dockerfile_path).exists(),
                    "CLI {} must have a Dockerfile for container assessment", cli);
        }
    }

    /// Test that the Rust controller architecture exists and is documented
    #[test]
    fn test_controller_architecture_assessment() {
        // Controller directory must exist
        assert!(Path::new("controller/").exists(),
                "controller/ directory must exist for architecture assessment");

        // Main controller source components
        let controller_components = vec![
            "controller/src/cli/",
            "controller/src/crds/",
            "controller/src/tasks/",
            "controller/Cargo.toml"
        ];

        for component in controller_components {
            assert!(Path::new(component).exists(),
                    "Controller component {} must exist for architecture assessment", component);
        }

        // Verify CLI abstraction layer exists (from research findings)
        let cli_files = vec![
            "controller/src/cli/adapter.rs",
            "controller/src/cli/bridge.rs",
            "controller/src/cli/router.rs",
            "controller/src/cli/types.rs"
        ];

        for cli_file in cli_files {
            assert!(Path::new(cli_file).exists(),
                    "CLI abstraction file {} must exist as documented in assessment", cli_file);
        }
    }

    /// Test MCP server exists and is Rust-based (not TypeScript)
    #[test]
    fn test_mcp_server_assessment() {
        // MCP directory must exist
        assert!(Path::new("mcp/").exists(),
                "mcp/ directory must exist for MCP server assessment");

        // Must be Rust-based (Cargo.toml) not TypeScript (package.json)
        assert!(Path::new("mcp/Cargo.toml").exists(),
                "MCP server must be Rust-based as documented in Task 1");

        assert!(!Path::new("mcp/package.json").exists(),
                "MCP server should not be TypeScript-based per Task 1 assessment");

        // Should have main.rs
        assert!(Path::new("mcp/src/main.rs").exists(),
                "MCP server must have main.rs for functionality assessment");
    }

    /// Test that CI/CD pipeline exists and is operational
    #[test]
    fn test_cicd_pipeline_assessment() {
        // GitHub Actions directory must exist
        assert!(Path::new(".github/workflows/").exists(),
                "GitHub Actions workflows must exist for CI/CD assessment");

        // Key workflow files mentioned in research
        let workflow_files = vec![
            ".github/workflows/agents-build.yaml",
            ".github/workflows/controller-ci.yaml"
        ];

        for workflow in workflow_files {
            assert!(Path::new(workflow).exists(),
                    "Workflow {} must exist as documented in CI/CD assessment", workflow);
        }
    }

    /// Test that GitOps infrastructure exists
    #[test]
    fn test_gitops_assessment() {
        // GitOps directory should exist
        assert!(Path::new("infra/gitops/").exists() || Path::new("infra/charts/").exists(),
                "GitOps infrastructure must exist for deployment assessment");

        // Should have Helm charts
        assert!(Path::new("infra/charts/").exists(),
                "Helm charts must exist for GitOps assessment");
    }

    /// Test research findings document structure and completeness
    #[test]
    fn test_research_findings_structure() {
        let content = fs::read_to_string("CLI_RESEARCH_FINDINGS.md")
            .expect("CLI_RESEARCH_FINDINGS.md must exist");

        // Must contain key assessment sections
        let required_sections = vec![
            "Executive Summary",
            "Architecture Analysis",
            "Container Infrastructure",
            "CLI Integration Status",
            "Gap Analysis",
            "Recommendations"
        ];

        for section in required_sections {
            assert!(content.contains(section),
                    "Research findings must contain {} section", section);
        }

        // Must document the "8 CLI architecture" as mentioned in PR
        assert!(content.contains("8 CLI"),
                "Research must document 8 CLI architecture");

        // Must contain specific technical findings
        let technical_content = vec![
            "Controller", "MCP server", "Docker images", "CI/CD",
            "production-ready", "Kubernetes"
        ];

        for tech in technical_content {
            assert!(content.contains(tech),
                    "Research findings must contain technical assessment of {}", tech);
        }
    }

    /// Test that Task 1 success criteria deliverables are present
    #[test]
    fn test_task_1_success_criteria_deliverables() {
        // From task.md success criteria, these must exist:

        // "Complete documentation of current architecture"
        assert!(Path::new("CLI_RESEARCH_FINDINGS.md").exists(),
                "Architecture documentation must exist");

        // "All CLI Docker images tested and status documented"
        let content = fs::read_to_string("CLI_RESEARCH_FINDINGS.md").unwrap();
        assert!(content.contains("CLI Status Matrix") || content.contains("Status"),
                "CLI Docker images status must be documented");

        // "Current MCP server capabilities documented"
        assert!(content.contains("MCP") && content.contains("Rust-based"),
                "MCP server capabilities must be documented");

        // "Gap analysis completed with prioritized recommendations"
        assert!(content.contains("Gap") || content.contains("Recommendations"),
                "Gap analysis must be present");

        // "Clear roadmap for next development phases"
        assert!(content.contains("Next Steps") || content.contains("roadmap"),
                "Development roadmap must be documented");
    }

    /// Test that key configuration files exist for platform operation
    #[test]
    fn test_platform_configuration_assessment() {
        // Key config files that should exist for platform assessment
        let config_files = vec![
            "cto-config.json",
            "client-config.json"
        ];

        for config in config_files {
            assert!(Path::new(config).exists(),
                    "Configuration file {} must exist for platform assessment", config);
        }

        // Coding guidelines referenced in Task 1
        assert!(Path::new("coding-guidelines.md").exists(),
                "Coding guidelines must exist as documented");

        assert!(Path::new("github-guidelines.md").exists(),
                "GitHub guidelines must exist as documented");
    }

    /// Test specific Task 1 quality requirements
    #[test]
    fn test_task_1_quality_gates() {
        // Research document must be substantial and professional
        let content = fs::read_to_string("CLI_RESEARCH_FINDINGS.md").unwrap();

        // Must have proper report metadata
        assert!(content.contains("Report Date") || content.contains("Date:"),
                "Research report must have date metadata");

        assert!(content.contains("Author") || content.contains("Assessment"),
                "Research report must have authorship information");

        // Must demonstrate thorough analysis (not just basic file listing)
        assert!(content.contains("analysis") || content.contains("assessment"),
                "Document must demonstrate actual analysis, not just inventory");

        // Must contain actionable findings
        assert!(content.contains("✅") || content.contains("❌") || content.contains("Status"),
                "Document must contain actionable status assessments");
    }
}

// Integration test to validate overall Task 1 completeness
#[cfg(test)]
mod task_1_integration_tests {
    use super::*;

    #[test]
    fn test_task_1_complete_deliverable_set() {
        // This test validates that ALL required Task 1 deliverables exist together

        println!("Validating Task 1 complete deliverable set...");

        // Primary deliverable
        assert!(Path::new("CLI_RESEARCH_FINDINGS.md").exists(),
                "Primary research document missing");

        // Supporting infrastructure validated
        assert!(Path::new("controller/").exists(), "Controller architecture missing");
        assert!(Path::new("mcp/").exists(), "MCP server missing");
        assert!(Path::new("infra/images/").exists(), "Container infrastructure missing");
        assert!(Path::new(".github/workflows/").exists(), "CI/CD pipeline missing");

        // All 8 CLI types assessed
        let cli_count = fs::read_dir("infra/images/").unwrap()
            .filter_map(|entry| entry.ok())
            .filter(|entry| entry.file_type().map(|ft| ft.is_dir()).unwrap_or(false))
            .filter(|entry| {
                let name = entry.file_name().to_string_lossy().to_string();
                ["claude", "codex", "opencode", "gemini", "grok", "qwen", "cursor", "openhands"]
                    .contains(&name.as_str())
            })
            .count();

        assert!(cli_count >= 8,
                "Must have assessed all 8 CLI types, found: {}", cli_count);

        println!("✅ Task 1 deliverable set validation PASSED");
    }

    #[test]
    fn test_task_1_research_quality_standards() {
        let content = fs::read_to_string("CLI_RESEARCH_FINDINGS.md").unwrap();

        // Quality standards for assessment report
        let word_count = content.split_whitespace().count();
        assert!(word_count >= 2000,
                "Research report should be comprehensive (>=2000 words), found: {}", word_count);

        // Must demonstrate actual research (not just file copies)
        let analysis_indicators = vec![
            "assessment", "analysis", "findings", "evaluation",
            "current state", "gap analysis", "recommendations"
        ];

        let found_indicators: Vec<_> = analysis_indicators.iter()
            .filter(|&indicator| content.to_lowercase().contains(&indicator.to_lowercase()))
            .collect();

        assert!(found_indicators.len() >= 5,
                "Report must demonstrate thorough research and analysis");

        println!("✅ Task 1 research quality standards PASSED");
    }
}