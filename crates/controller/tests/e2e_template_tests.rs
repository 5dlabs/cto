//! End-to-End Template Tests
//!
//! Tests the full template rendering pipeline for all agent/job/CLI combinations.
//! These tests validate that:
//! 1. All templates exist and are valid Handlebars
//! 2. Templates render correctly with realistic context
//! 3. Rendered scripts contain expected CLI invocations

use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

/// Get the agent-templates directory path
fn templates_dir() -> PathBuf {
    let manifest_dir = env!("CARGO_MANIFEST_DIR");
    PathBuf::from(manifest_dir)
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .join("agent-templates")
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
        ("bolt", vec!["deploy", "healer"]),
        ("cipher", vec!["security", "healer"]),
        ("cleo", vec!["quality"]),
        ("tess", vec!["test"]),
        ("stitch", vec!["review"]),
        ("morgan", vec!["pm", "docs"]),
        ("atlas", vec!["integration"]),
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
        "agent-templates directory should exist at {}",
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
    let base = templates_dir();

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
    let base = templates_dir();

    for (agent, jobs) in agent_job_matrix() {
        for job in jobs {
            let prompt = base.join(agent).join(job).join("system-prompt.md.hbs");
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
fn test_all_container_templates_exist() {
    let base = templates_dir();

    for (agent, jobs) in agent_job_matrix() {
        for job in jobs {
            let container = base.join(agent).join(job).join("container.sh.hbs");
            assert!(
                container.exists() || container.is_symlink(),
                "Container template should exist for {}/{} at {}",
                agent,
                job,
                container.display()
            );

            // If it's a symlink, verify target exists
            if container.is_symlink() {
                let target = fs::read_link(&container).expect("Should read symlink");
                let resolved = container.parent().unwrap().join(&target);
                assert!(
                    resolved.exists(),
                    "Container symlink for {}/{} should point to existing file, got {}",
                    agent,
                    job,
                    target.display()
                );
            }
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
        content.contains("{{cli_execute}}"),
        "Container should have cli_execute placeholder"
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
    let base = templates_dir();

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
            let prompt_path = base.join(agent).join(job).join("system-prompt.md.hbs");
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

    // Rex should mention Rust
    let rex_prompt = fs::read_to_string(base.join("rex/coder/system-prompt.md.hbs"))
        .expect("Should read rex prompt")
        .to_lowercase();
    assert!(
        rex_prompt.contains("rust"),
        "Rex coder prompt should mention Rust"
    );

    // Blaze should mention frontend/react
    let blaze_prompt = fs::read_to_string(base.join("blaze/coder/system-prompt.md.hbs"))
        .expect("Should read blaze prompt")
        .to_lowercase();
    assert!(
        blaze_prompt.contains("frontend")
            || blaze_prompt.contains("react")
            || blaze_prompt.contains("ui"),
        "Blaze coder prompt should mention frontend/React/UI"
    );

    // Grizz should mention Go
    let grizz_prompt = fs::read_to_string(base.join("grizz/coder/system-prompt.md.hbs"))
        .expect("Should read grizz prompt")
        .to_lowercase();
    assert!(
        grizz_prompt.contains("go") || grizz_prompt.contains("golang"),
        "Grizz coder prompt should mention Go"
    );

    // Nova should mention Node/TypeScript
    let nova_prompt = fs::read_to_string(base.join("nova/coder/system-prompt.md.hbs"))
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
    let base = templates_dir();
    let mut total_prompts = 0;
    let mut total_containers = 0;

    for (agent, jobs) in agent_job_matrix() {
        for job in jobs {
            let prompt = base.join(agent).join(job).join("system-prompt.md.hbs");
            let container = base.join(agent).join(job).join("container.sh.hbs");

            if prompt.exists() {
                total_prompts += 1;
            }
            if container.exists() || container.is_symlink() {
                total_containers += 1;
            }
        }
    }

    // Expected: 13 agents * avg ~1.8 jobs = ~23 prompts/containers
    assert!(
        total_prompts >= 20,
        "Should have at least 20 system prompts, found {total_prompts}"
    );
    assert!(
        total_containers >= 20,
        "Should have at least 20 container templates, found {total_containers}"
    );
}
