//! End-to-End Template Tests
//!
//! Tests the full template rendering pipeline for all agent/job/CLI combinations.
//! These tests validate that:
//! 1. All templates exist and are valid Handlebars
//! 2. Templates render correctly with realistic context
//! 3. Rendered scripts contain expected CLI invocations

#![allow(clippy::doc_markdown)] // Env vars and type names in doc comments are fine

use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

/// Get the templates directory path
fn templates_dir() -> PathBuf {
    let manifest_dir = env!("CARGO_MANIFEST_DIR");
    PathBuf::from(manifest_dir)
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .join("templates")
}

/// Get the agents directory path (templates/agents/)
fn agents_dir() -> PathBuf {
    templates_dir().join("agents")
}

/// Agent and their supported job types
fn agent_job_matrix() -> Vec<(&'static str, Vec<&'static str>)> {
    vec![
        ("rex", vec!["coder", "healer"]),
        ("blaze", vec!["coder", "healer"]),
        ("grizz", vec!["coder", "healer"]),
        ("nova", vec!["coder", "healer", "docs"]),
        ("tap", vec!["coder", "healer"]),
        ("spark", vec!["coder", "healer"]),
        ("bolt", vec!["infra", "deploy", "healer"]),
        ("cipher", vec!["security", "healer"]),
        ("cleo", vec!["quality"]),
        ("tess", vec!["test"]),
        ("stitch", vec!["review"]),
        ("morgan", vec!["pm", "docs", "intake"]),
        ("atlas", vec!["integration", "intake"]),
    ]
}

/// Supported CLIs
#[allow(dead_code)]
fn cli_types() -> Vec<&'static str> {
    vec!["claude", "codex", "opencode", "cursor", "factory", "gemini"]
}

#[test]
fn test_templates_dir_exists() {
    let dir = templates_dir();
    assert!(
        dir.exists(),
        "templates directory should exist at {}",
        dir.display()
    );
}

#[test]
fn test_shared_container_template_exists() {
    let template = templates_dir().join("_shared/container.sh.hbs");
    assert!(
        template.exists(),
        "Shared container template should exist at {}",
        template.display()
    );
}

#[test]
fn test_all_partials_exist() {
    let partials = vec![
        "header.sh.hbs",
        "rust-env.sh.hbs",
        "go-env.sh.hbs",
        "node-env.sh.hbs",
        "config.sh.hbs",
        "github-auth.sh.hbs",
        "git-setup.sh.hbs",
        "task-files.sh.hbs",
        "tools-config.sh.hbs",
        "acceptance-probe.sh.hbs",
        "completion.sh.hbs",
    ];

    let partials_dir = templates_dir().join("_shared/partials");
    assert!(partials_dir.exists(), "Partials directory should exist");

    for partial in partials {
        let path = partials_dir.join(partial);
        assert!(
            path.exists(),
            "Partial {} should exist at {}",
            partial,
            path.display()
        );
    }
}

#[test]
fn test_all_agent_directories_exist() {
    let base = agents_dir();

    for (agent, _) in agent_job_matrix() {
        let agent_dir = base.join(agent);
        assert!(
            agent_dir.exists(),
            "Agent directory should exist for {} at {}",
            agent,
            agent_dir.display()
        );
    }
}

#[test]
fn test_all_system_prompts_exist() {
    let base = agents_dir();

    for (agent, jobs) in agent_job_matrix() {
        for job in jobs {
            // New flat structure: agents/{agent}/{job}.md.hbs
            let prompt = base.join(agent).join(format!("{job}.md.hbs"));
            assert!(
                prompt.exists(),
                "System prompt should exist for {}/{} at {}",
                agent,
                job,
                prompt.display()
            );

            // Verify it's not empty
            let content = fs::read_to_string(&prompt).expect("Should read prompt");
            assert!(
                !content.is_empty(),
                "System prompt for {agent}/{job} should not be empty"
            );

            // Verify it contains expected content
            assert!(
                content.contains('#') || content.contains("{{"),
                "System prompt for {agent}/{job} should contain markdown headers or template vars"
            );
        }
    }
}

#[test]
fn test_container_templates_consolidated() {
    // After consolidation, only morgan/intake has a unique container template.
    // All other agents use the shared _shared/container.sh.hbs template.
    // Agent-specific behavior is handled via:
    //   1. job_type conditionals in the shared template (e.g., infra setup for bolt)
    //   2. Agent-specific system prompts
    // Flat structure: agents/{agent}/{job}.sh.hbs for container templates
    let base = agents_dir();

    // Verify the only agent-specific container template is morgan/intake.sh.hbs
    let morgan_intake_container = base.join("morgan/intake.sh.hbs");
    assert!(
        morgan_intake_container.exists(),
        "Morgan intake container should exist at {}",
        morgan_intake_container.display()
    );

    // Verify no other container templates exist (they were consolidated)
    for (agent, jobs) in agent_job_matrix() {
        for job in jobs {
            // Skip morgan/intake which is the only exception
            if agent == "morgan" && job == "intake" {
                continue;
            }

            // Check both old nested format and new flat format don't exist
            let container_old = base.join(agent).join(job).join("container.sh.hbs");
            let container_new = base.join(agent).join(format!("{job}.sh.hbs"));
            assert!(
                !container_old.exists() && !container_old.is_symlink(),
                "Old container template should NOT exist for {}/{} at {}",
                agent,
                job,
                container_old.display()
            );
            assert!(
                !container_new.exists() && !container_new.is_symlink(),
                "Container template should NOT exist for {}/{} - use shared template instead. Found at {}",
                agent,
                job,
                container_new.display()
            );
        }
    }
}

#[test]
fn test_shared_container_renders() {
    let template_path = templates_dir().join("_shared/container.sh.hbs");
    let content = fs::read_to_string(&template_path).expect("Should read template");

    // Check for partial references
    assert!(content.contains("{{>"), "Container should use partials");
    assert!(
        content.contains("{{> header"),
        "Container should include header partial"
    );
    assert!(
        content.contains("{{> config"),
        "Container should include config partial"
    );
    assert!(
        content.contains("{{> github-auth"),
        "Container should include github-auth partial"
    );
    assert!(
        content.contains("{{> git-setup"),
        "Container should include git-setup partial"
    );
    assert!(
        content.contains("{{> cli_execute}}"),
        "Container should have cli_execute partial"
    );
}

#[test]
fn test_partials_are_valid_handlebars() {
    let partials_dir = templates_dir().join("_shared/partials");

    for entry in fs::read_dir(&partials_dir).expect("Should read partials dir") {
        let entry = entry.expect("Should read entry");
        let path = entry.path();

        if path.extension().is_some_and(|e| e == "hbs") {
            let content = fs::read_to_string(&path).expect("Should read partial");

            // Basic validation - check balanced braces
            let open_count = content.matches("{{").count();
            let close_count = content.matches("}}").count();

            assert_eq!(
                open_count,
                close_count,
                "Partial {} should have balanced Handlebars braces",
                path.file_name().unwrap().to_string_lossy()
            );
        }
    }
}

#[test]
fn test_system_prompts_contain_role_context() {
    let base = agents_dir();

    let role_keywords: HashMap<&str, Vec<&str>> = [
        ("coder", vec!["code", "implement", "develop", "feature"]),
        (
            "healer",
            vec!["fix", "diagnose", "issue", "error", "incident"],
        ),
        ("deploy", vec!["deploy", "release", "ci", "cd", "pipeline"]),
        ("security", vec!["security", "vulnerab", "scan", "audit"]),
        ("quality", vec!["quality", "review", "standard", "lint"]),
        ("test", vec!["test", "qa", "coverage", "spec"]),
        ("review", vec!["review", "pr", "pull request", "feedback"]),
        ("pm", vec!["product", "spec", "requirement", "user"]),
        ("docs", vec!["document", "doc", "readme", "guide"]),
        (
            "integration",
            vec!["merge", "conflict", "integrate", "rebase"],
        ),
    ]
    .into_iter()
    .collect();

    for (agent, jobs) in agent_job_matrix() {
        for job in jobs {
            // Flat structure: agents/{agent}/{job}.md.hbs
            let prompt_path = base.join(agent).join(format!("{job}.md.hbs"));
            let content = fs::read_to_string(&prompt_path)
                .expect("Should read prompt")
                .to_lowercase();

            if let Some(keywords) = role_keywords.get(job) {
                let found_any = keywords.iter().any(|kw| content.contains(kw));
                assert!(
                    found_any,
                    "System prompt for {agent}/{job} should contain at least one role keyword: {keywords:?}"
                );
            }
        }
    }
}

#[test]
fn test_agent_specialization_in_prompts() {
    let base = templates_dir();

    // Rex should mention Rust (flat structure: agents/{agent}/{job}.md.hbs)
    let rex_prompt = fs::read_to_string(base.join("agents/rex/coder.md.hbs"))
        .expect("Should read rex prompt")
        .to_lowercase();
    assert!(
        rex_prompt.contains("rust"),
        "Rex coder prompt should mention Rust"
    );

    // Blaze should mention frontend/react
    let blaze_prompt = fs::read_to_string(base.join("agents/blaze/coder.md.hbs"))
        .expect("Should read blaze prompt")
        .to_lowercase();
    assert!(
        blaze_prompt.contains("frontend")
            || blaze_prompt.contains("react")
            || blaze_prompt.contains("ui"),
        "Blaze coder prompt should mention frontend/React/UI"
    );

    // Grizz should mention Go
    let grizz_prompt = fs::read_to_string(base.join("agents/grizz/coder.md.hbs"))
        .expect("Should read grizz prompt")
        .to_lowercase();
    assert!(
        grizz_prompt.contains("go") || grizz_prompt.contains("golang"),
        "Grizz coder prompt should mention Go"
    );

    // Nova should mention Node/TypeScript
    let nova_prompt = fs::read_to_string(base.join("agents/nova/coder.md.hbs"))
        .expect("Should read nova prompt")
        .to_lowercase();
    assert!(
        nova_prompt.contains("node")
            || nova_prompt.contains("typescript")
            || nova_prompt.contains("javascript"),
        "Nova coder prompt should mention Node/TypeScript"
    );
}

#[test]
fn test_runtime_partials_exist() {
    let partials_dir = templates_dir().join("_shared/partials");

    // Rust runtime
    let rust_env = partials_dir.join("rust-env.sh.hbs");
    let rust_content = fs::read_to_string(&rust_env).expect("Should read rust-env");
    assert!(
        rust_content.contains("cargo") || rust_content.contains("rustup"),
        "Rust env should set up cargo/rustup"
    );

    // Go runtime
    let go_env = partials_dir.join("go-env.sh.hbs");
    let go_content = fs::read_to_string(&go_env).expect("Should read go-env");
    assert!(
        go_content.contains("GOPATH") || go_content.contains("go"),
        "Go env should set up GOPATH"
    );

    // Node runtime
    let node_env = partials_dir.join("node-env.sh.hbs");
    let node_content = fs::read_to_string(&node_env).expect("Should read node-env");
    assert!(
        node_content.contains("nvm") || node_content.contains("node"),
        "Node env should set up nvm/node"
    );
}

#[test]
fn test_github_auth_partial_security() {
    let partial = templates_dir().join("_shared/partials/github-auth.sh.hbs");
    let content = fs::read_to_string(&partial).expect("Should read github-auth");

    // Should not echo tokens
    assert!(
        !content.contains("echo $GH_TOKEN") && !content.contains("echo $GITHUB_TOKEN"),
        "GitHub auth should not echo tokens to stdout"
    );

    // Should export necessary vars
    assert!(
        content.contains("GIT_AUTHOR_NAME"),
        "GitHub auth should set GIT_AUTHOR_NAME"
    );
    assert!(
        content.contains("GIT_AUTHOR_EMAIL"),
        "GitHub auth should set GIT_AUTHOR_EMAIL"
    );
}

#[test]
fn test_template_count_matches_expected() {
    let base = agents_dir();
    let mut total_prompts = 0;

    for (agent, jobs) in agent_job_matrix() {
        for job in jobs {
            // Flat structure: agents/{agent}/{job}.md.hbs
            let prompt = base.join(agent).join(format!("{job}.md.hbs"));

            if prompt.exists() {
                total_prompts += 1;
            }
        }
    }

    // Expected: 13 agents * avg ~2 jobs = ~26 system prompts
    // Container templates are consolidated to _shared/container.sh.hbs
    // (only morgan/intake has a unique container template)
    assert!(
        total_prompts >= 20,
        "Should have at least 20 system prompts, found {total_prompts}"
    );

    // Verify the shared container template exists
    let shared_container = templates_dir().join("_shared/container.sh.hbs");
    assert!(
        shared_container.exists(),
        "Shared container template should exist"
    );

    // Verify the only unique container (morgan/intake.sh.hbs) exists
    let morgan_container = base.join("morgan/intake.sh.hbs");
    assert!(
        morgan_container.exists(),
        "Morgan intake container should exist"
    );
}

// ============================================================================
// Full Template Rendering Tests
// These tests verify that templates render correctly through CodeTemplateGenerator
// ============================================================================

#[allow(clippy::disallowed_macros)] // println! is fine in tests for output
mod rendering_tests {
    use controller::cli::types::CLIType;
    use controller::crds::coderun::CLIConfig;
    use controller::crds::{CodeRun, CodeRunSpec};
    use controller::tasks::code::templates::CodeTemplateGenerator;
    use controller::tasks::config::ControllerConfig;
    use k8s_openapi::apimachinery::pkg::apis::meta::v1::ObjectMeta;
    use std::collections::HashMap;
    use std::path::PathBuf;

    /// Check if templates are available for testing via AGENT_TEMPLATES_PATH
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

    /// Create a test CodeRun with specific agent and CLI type
    fn create_code_run(github_app: &str, cli_type: CLIType) -> CodeRun {
        let mut settings = HashMap::new();
        settings.insert("approvalPolicy".to_string(), serde_json::json!("never"));
        settings.insert(
            "sandboxMode".to_string(),
            serde_json::json!("workspace-write"),
        );

        CodeRun {
            metadata: ObjectMeta {
                name: Some("test-run".to_string()),
                namespace: Some("default".to_string()),
                ..Default::default()
            },
            spec: CodeRunSpec {
                run_type: "implementation".to_string(),
                cli_config: Some(CLIConfig {
                    cli_type,
                    model: "test-model".to_string(),
                    settings,
                    max_tokens: Some(16000),
                    temperature: Some(0.7),
                    model_rotation: None,
                }),
                task_id: Some(1),
                service: "test-service".to_string(),
                repository_url: "https://github.com/test/repo".to_string(),
                docs_repository_url: "https://github.com/test/docs".to_string(),
                docs_project_directory: Some("docs".to_string()),
                working_directory: Some("src".to_string()),
                model: "test-model".to_string(),
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
                prompt_modification: None,
                acceptance_criteria: None,
            },
            status: None,
        }
    }

    /// Agent/CLI combinations to test
    fn agent_cli_matrix() -> Vec<(&'static str, CLIType, &'static str)> {
        vec![
            // Rex (Rust) with all CLIs
            ("5DLabs-Rex", CLIType::Claude, "claude"),
            ("5DLabs-Rex", CLIType::Codex, "codex"),
            ("5DLabs-Rex", CLIType::Cursor, "cursor"),
            ("5DLabs-Rex", CLIType::Factory, "factory"),
            ("5DLabs-Rex", CLIType::Gemini, "gemini"),
            ("5DLabs-Rex", CLIType::OpenCode, "opencode"),
            // Blaze (Frontend) with Claude
            ("5DLabs-Blaze", CLIType::Claude, "claude"),
            // Cleo (Quality) with Claude
            ("5DLabs-Cleo", CLIType::Claude, "claude"),
            // Tess (Test) with Claude
            ("5DLabs-Tess", CLIType::Claude, "claude"),
            // Atlas (Integration) with Claude
            ("5DLabs-Atlas", CLIType::Claude, "claude"),
            // Bolt (Deploy) with Claude
            ("5DLabs-Bolt", CLIType::Claude, "claude"),
        ]
    }

    /// CLI-specific invocation markers to verify in rendered output
    fn cli_markers(cli_type: CLIType) -> Vec<&'static str> {
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

    #[test]
    fn test_container_script_renders_for_all_agent_cli_combos() {
        skip_if_no_templates!();
        let config = ControllerConfig::default();

        for (agent, cli_type, cli_name) in agent_cli_matrix() {
            let code_run = create_code_run(agent, cli_type);

            // Generate all templates
            let result = CodeTemplateGenerator::generate_all_templates(&code_run, &config);

            assert!(
                result.is_ok(),
                "Template generation failed for {agent} + {cli_name}: {:?}",
                result.err()
            );

            let templates = result.unwrap();

            // Verify container.sh was generated
            assert!(
                templates.contains_key("container.sh"),
                "container.sh should be generated for {agent} + {cli_name}"
            );

            let container_script = &templates["container.sh"];

            // Verify it's not empty
            assert!(
                !container_script.is_empty(),
                "container.sh should not be empty for {agent} + {cli_name}"
            );

            // Verify it contains shell script markers
            assert!(
                container_script.contains("#!/bin/bash") || container_script.contains("set -"),
                "container.sh should be a bash script for {agent} + {cli_name}"
            );

            // Verify it contains CLI-specific invocation
            let markers = cli_markers(cli_type);
            let found_cli_marker = markers.iter().any(|m| container_script.contains(m));
            assert!(
                found_cli_marker,
                "container.sh for {agent} + {cli_name} should contain CLI marker from {markers:?}"
            );

            // Verify common sections exist
            assert!(
                container_script.contains("GIT_") || container_script.contains("git"),
                "container.sh should have git setup for {agent} + {cli_name}"
            );

            println!("✓ {agent} + {cli_name}: {} bytes", container_script.len());
        }
    }

    #[test]
    fn test_system_prompt_renders_for_agents() {
        skip_if_no_templates!();
        let config = ControllerConfig::default();

        let agents = vec![
            "5DLabs-Rex",
            "5DLabs-Blaze",
            "5DLabs-Cleo",
            "5DLabs-Tess",
            "5DLabs-Atlas",
            "5DLabs-Bolt",
        ];

        for agent in agents {
            let code_run = create_code_run(agent, CLIType::Claude);
            let result = CodeTemplateGenerator::generate_all_templates(&code_run, &config);

            assert!(
                result.is_ok(),
                "Template generation failed for {agent}: {:?}",
                result.err()
            );

            let templates = result.unwrap();

            // Check for memory file (CLAUDE.md for Claude CLI)
            let memory_key = "CLAUDE.md";
            assert!(
                templates.contains_key(memory_key),
                "{memory_key} should be generated for {agent}"
            );

            let memory = &templates[memory_key];
            assert!(
                !memory.is_empty(),
                "{memory_key} should not be empty for {agent}"
            );

            println!("✓ {agent} memory: {} bytes", memory.len());
        }
    }

    #[test]
    fn test_config_generates_for_all_cli_types() {
        skip_if_no_templates!();
        let config = ControllerConfig::default();

        // Each CLI generates a different config file
        // Claude: settings.json, mcp.json
        // Codex: codex-config.toml
        // Cursor: cursor-cli-config.json, cursor-cli.json, cursor-mcp.json
        // Factory: factory-cli-config.json, factory-cli.json
        // Gemini: settings.json
        // OpenCode: opencode-config.json
        let cli_configs: Vec<(CLIType, &str)> = vec![
            (CLIType::Claude, "settings.json"),
            (CLIType::Codex, "codex-config.toml"),
            (CLIType::Cursor, "cursor-cli-config.json"),
            (CLIType::Factory, "factory-cli-config.json"),
            (CLIType::Gemini, "settings.json"),
            (CLIType::OpenCode, "opencode-config.json"),
        ];

        for (cli_type, config_file) in cli_configs {
            let code_run = create_code_run("5DLabs-Rex", cli_type);
            let result = CodeTemplateGenerator::generate_all_templates(&code_run, &config);

            assert!(
                result.is_ok(),
                "Template generation failed for CLI {cli_type:?}: {:?}",
                result.err()
            );

            let templates = result.unwrap();

            // Verify config file exists
            assert!(
                templates.contains_key(config_file),
                "{config_file} should be generated for CLI {cli_type:?}"
            );

            let config_content = &templates[config_file];
            assert!(
                !config_content.is_empty(),
                "{config_file} should not be empty for CLI {cli_type:?}"
            );

            println!(
                "✓ {cli_type:?} config ({config_file}): {} bytes",
                config_content.len()
            );
        }
    }
}
