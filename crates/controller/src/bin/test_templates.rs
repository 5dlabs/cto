#!/usr/bin/env cargo
//! Template testing utility for local handlebars template validation
//!
//! Usage: cargo run --bin `test_templates`

#![allow(clippy::disallowed_macros)]

use controller::tasks::template_paths::{
    CODE_CLAUDE_CONTAINER_TEMPLATE, CODE_CLAUDE_MEMORY_TEMPLATE, CODE_CODEX_AGENTS_TEMPLATE,
    CODE_CODEX_CONTAINER_BASE_TEMPLATE, CODE_CODEX_CONTAINER_TEMPLATE, CODE_CURSOR_AGENTS_TEMPLATE,
    CODE_CURSOR_CONTAINER_BASE_TEMPLATE, CODE_CURSOR_CONTAINER_TEMPLATE,
    CODE_FACTORY_AGENTS_TEMPLATE, CODE_FACTORY_CONTAINER_BASE_TEMPLATE,
    CODE_FACTORY_CONTAINER_TEMPLATE, SHARED_BOOTSTRAP_RUST_ENV, SHARED_CONTAINER_CORE,
    SHARED_FUNCTIONS_COMPLETION_MARKER, SHARED_FUNCTIONS_DOCKER_SIDECAR, SHARED_FUNCTIONS_GH_CLI,
    SHARED_FUNCTIONS_GITHUB_AUTH, SHARED_FUNCTIONS_GIT_OPERATIONS, SHARED_FUNCTIONS_QUALITY_GATES,
    SHARED_PROMPTS_CONTEXT7, SHARED_PROMPTS_DESIGN_SYSTEM,
};
use handlebars::Handlebars;
use serde_json::json;
use std::path::Path;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🧪 Testing Handlebars Templates...\n");

    // Initialize handlebars engine
    let mut handlebars = Handlebars::new();

    // Template directory - relative path from controller directory to infra/charts/controller/templates
    // Navigate from CARGO_MANIFEST_DIR (crates/controller) up to workspace root
    let template_dir =
        Path::new(env!("CARGO_MANIFEST_DIR")).join("../../infra/charts/controller/templates");

    // Register shared partials first (these are used by CLI-specific templates)
    register_shared_partials(&mut handlebars, &template_dir)?;

    // Test code templates
    test_code_templates(&mut handlebars, &template_dir)?;

    println!("✅ All templates rendered successfully!");
    Ok(())
}

/// Register shared partials that are used across all CLI templates
fn register_shared_partials(
    handlebars: &mut Handlebars,
    template_dir: &Path,
) -> Result<(), Box<dyn std::error::Error>> {
    println!("📦 Registering shared partials:");

    // Map partial name (used in templates) -> template path
    let shared_partials = vec![
        ("shared/bootstrap/rust-env", SHARED_BOOTSTRAP_RUST_ENV),
        ("shared/functions/github-auth", SHARED_FUNCTIONS_GITHUB_AUTH),
        (
            "shared/functions/docker-sidecar",
            SHARED_FUNCTIONS_DOCKER_SIDECAR,
        ),
        (
            "shared/functions/completion-marker",
            SHARED_FUNCTIONS_COMPLETION_MARKER,
        ),
        (
            "shared/functions/git-operations",
            SHARED_FUNCTIONS_GIT_OPERATIONS,
        ),
        ("shared/functions/gh-cli", SHARED_FUNCTIONS_GH_CLI),
        (
            "shared/functions/quality-gates",
            SHARED_FUNCTIONS_QUALITY_GATES,
        ),
        ("shared/context7-instructions", SHARED_PROMPTS_CONTEXT7),
        ("shared/design-system", SHARED_PROMPTS_DESIGN_SYSTEM),
        ("shared/container-core", SHARED_CONTAINER_CORE),
    ];

    for (partial_name, template_path) in shared_partials {
        let full_path = template_dir.join(template_path);
        if full_path.exists() {
            let content = std::fs::read_to_string(&full_path)?;
            handlebars.register_partial(partial_name, content)?;
            println!("  ✅ {partial_name}");
        } else {
            println!("  ⚠️  Partial not found: {template_path}");
        }
    }
    println!();

    Ok(())
}

#[allow(clippy::too_many_lines)]
fn test_code_templates(
    handlebars: &mut Handlebars,
    template_dir: &Path,
) -> Result<(), Box<dyn std::error::Error>> {
    println!("💻 Testing Code Templates:");

    // Mock Claude data
    let claude_data = json!({
        "task_id": 42,
        "service": "simple-api",
        "repository_url": "https://github.com/5dlabs/cto",
        "platform_repository_url": "https://github.com/5dlabs/cto",
        "branch": "feature/example-project-and-cli",
        "working_directory": "_projects/simple-api",
        "model": "claude-sonnet-4-5-20250929",
        "github_user": "pm0-5dlabs",
        "local_tools": "bash,edit,read",
        "remote_tools": "github_create_issue",
        "tool_config": "default",
        "context_version": 1,
        "prompt_modification": null,
        "prompt_mode": "append"
    });

    for template_name in [CODE_CLAUDE_MEMORY_TEMPLATE, CODE_CLAUDE_CONTAINER_TEMPLATE] {
        let template_path = template_dir.join(template_name);

        if template_path.exists() {
            println!("  Testing {template_name}...");
            let template_content = std::fs::read_to_string(&template_path)?;
            handlebars.register_template_string(template_name, &template_content)?;
            let result = handlebars.render(template_name, &claude_data)?;
            println!("    ✅ Rendered successfully ({} chars)", result.len());
            for line in result.lines().take(3) {
                println!("    │ {line}");
            }
            if result.lines().count() > 3 {
                println!("    │ ... ({} total lines)", result.lines().count());
            }
            println!();
        } else {
            println!("  ⚠️  Template not found: {}", template_path.display());
        }
    }

    // Mock Codex data
    let codex_data = json!({
        "task_id": 42,
        "service": "simple-api",
        "repository_url": "https://github.com/5dlabs/cto",
        "docs_repository_url": "https://github.com/5dlabs/cto-docs",
        "docs_branch": "main",
        "docs_project_directory": "_projects/simple-api",
        "working_directory": "simple-api",
        "continue_session": false,
        "overwrite_memory": false,
        "github_app": "5DLabs-Rex",
        "cli": {
            "type": "codex",
            "model": "gpt-5-codex",
            "settings": json!({"sandboxPreset": "workspace-write"}),
            "remote_tools": ["memory_create_entities"]
        },
        "cli_config": {
            "cliType": "codex",
            "model": "gpt-5-codex",
            "maxTokens": 64000,
            "temperature": 0.7,
            "settings": {
                "sandboxPreset": "workspace-write",
                "approvalPolicy": "never"
            }
        },
        "model": "gpt-5-codex",
        "temperature": 0.7,
        "max_output_tokens": 64000,
        "approval_policy": "never",
        "sandbox_mode": "workspace-write",
        "project_doc_max_bytes": 32768,
        "tools": {
            "url": "http://tools.test",
            "tools": ["memory_create_entities"]
        },
        "model_provider": {
            "name": "OpenAI",
            "base_url": "https://api.openai.com/v1",
            "env_key": "OPENAI_API_KEY",
            "wire_api": "chat"
        },
        "client_config": {
            "remoteTools": ["memory_create_entities"],
            "localServers": {}
        }
    });

    // Ensure Codex partials are available before rendering templates that depend on them
    let codex_base_template_path = template_dir.join(CODE_CODEX_CONTAINER_BASE_TEMPLATE);
    if codex_base_template_path.exists() {
        let base_template_content = std::fs::read_to_string(&codex_base_template_path)?;
        handlebars.register_partial("codex_container_base", base_template_content)?;
    } else {
        println!(
            "  ⚠️  Codex base container partial missing: {}",
            codex_base_template_path.display()
        );
    }

    for template_name in [CODE_CODEX_CONTAINER_TEMPLATE, CODE_CODEX_AGENTS_TEMPLATE] {
        let template_path = template_dir.join(template_name);

        if template_path.exists() {
            println!("  Testing {template_name}...");
            let template_content = std::fs::read_to_string(&template_path)?;
            handlebars.register_template_string(template_name, &template_content)?;
            let result = handlebars.render(template_name, &codex_data)?;
            println!("    ✅ Rendered successfully ({} chars)", result.len());
            for line in result.lines().take(3) {
                println!("    │ {line}");
            }
            if result.lines().count() > 3 {
                println!("    │ ... ({} total lines)", result.lines().count());
            }
            println!();
        } else {
            println!("  ⚠️  Template not found: {}", template_path.display());
        }
    }

    let cursor_data = json!({
        "task_id": 42,
        "service": "simple-api",
        "repository_url": "https://github.com/5dlabs/cto",
        "docs_repository_url": "https://github.com/5dlabs/cto-docs",
        "docs_branch": "main",
        "docs_project_directory": "_projects/simple-api",
        "working_directory": "simple-api",
        "continue_session": false,
        "overwrite_memory": false,
        "github_app": "5DLabs-Rex",
        "workflow_name": "play-task-42-workflow",
        "cli": {
            "type": "cursor",
            "model": "gpt-5-cursor",
            "settings": json!({
                "sandboxMode": "danger-full-access",
                "approvalPolicy": "never",
                "editor": { "vimMode": true }
            }),
            "remote_tools": [
                "memory_create_entities",
                "brave_search_brave_web_search"
            ]
        },
        "cli_config": {
            "cliType": "cursor",
            "model": "gpt-5-cursor",
            "maxTokens": 64000,
            "temperature": 0.7,
            "settings": {
                "sandboxMode": "danger-full-access",
                "approvalPolicy": "never",
                "editor": { "vimMode": true }
            }
        },
        "model": "gpt-5-cursor",
        "temperature": 0.7,
        "max_output_tokens": 64000,
        "approval_policy": "never",
        "sandbox_mode": "danger-full-access",
        "project_doc_max_bytes": 32768,
        "editor_vim_mode": true,
        "tools": {
            "url": "http://tools.test",
            "tools": [
                "memory_create_entities",
                "brave_search_brave_web_search"
            ]
        },
        "raw_additional_json": "{}",
        "client_config": {
            "remoteTools": [
                "memory_create_entities",
                "brave_search_brave_web_search"
            ],
            "localServers": {}
        }
    });

    let cursor_base_template_path = template_dir.join(CODE_CURSOR_CONTAINER_BASE_TEMPLATE);
    if cursor_base_template_path.exists() {
        let base_template_content = std::fs::read_to_string(&cursor_base_template_path)?;
        handlebars.register_partial("cursor_container_base", base_template_content)?;
    } else {
        println!(
            "  ⚠️  Cursor base container partial missing: {}",
            cursor_base_template_path.display()
        );
    }

    for template_name in [CODE_CURSOR_CONTAINER_TEMPLATE, CODE_CURSOR_AGENTS_TEMPLATE] {
        let template_path = template_dir.join(template_name);

        if template_path.exists() {
            println!("  Testing {template_name}...");
            let template_content = std::fs::read_to_string(&template_path)?;
            handlebars.register_template_string(template_name, &template_content)?;
            let result = handlebars.render(template_name, &cursor_data)?;
            println!("    ✅ Rendered successfully ({} chars)", result.len());
            for line in result.lines().take(3) {
                println!("    │ {line}");
            }
            if result.lines().count() > 3 {
                println!("    │ ... ({} total lines)", result.lines().count());
            }
            println!();
        } else {
            println!("  ⚠️  Template not found: {}", template_path.display());
        }
    }

    let factory_data = json!({
        "task_id": 42,
        "service": "simple-api",
        "repository_url": "https://github.com/5dlabs/cto",
        "docs_repository_url": "https://github.com/5dlabs/cto-docs",
        "docs_branch": "main",
        "docs_project_directory": "_projects/simple-api",
        "working_directory": "simple-api",
        "continue_session": false,
        "overwrite_memory": false,
        "github_app": "5DLabs-Rex",
        "workflow_name": "play-task-42-workflow",
        "model": "gpt-5-factory",
        "cli": {
            "type": "factory",
            "model": "gpt-5-factory",
            "settings": json!({
                "approvalPolicy": "never",
                "sandboxMode": "danger-full-access",
                "projectDocMaxBytes": 65536,
                "reasoningEffort": "high",
                "toolsUrl": "http://tools.test",
                "editor": { "vimMode": true },
                "modelProvider": {
                    "name": "Factory",
                    "baseUrl": "https://api.factory.ai/v1",
                    "envKey": "FACTORY_API_KEY",
                    "wireApi": "chat"
                },
                "rawJson": "{\"extra\":\"value\"}",
                "modelRotation": [
                    "claude-sonnet-4-5-20250929",
                    "gpt-5-codex"
                ],
                "listToolsOnStart": true
            }),
            "remote_tools": [
                "memory_create_entities",
                "brave_search_brave_web_search"
            ]
        },
        "cli_config": {
            "cliType": "factory",
            "model": "gpt-5-factory",
            "maxTokens": 64000,
            "temperature": 0.5,
            "reasoningEffort": "high",
            "modelRotation": [
                "claude-sonnet-4-5-20250929",
                "gpt-5-codex"
            ],
            "listToolsOnStart": true,
            "settings": json!({
                "approvalPolicy": "never",
                "sandboxMode": "danger-full-access",
                "projectDocMaxBytes": 65536,
                "reasoningEffort": "high",
                "toolsUrl": "http://tools.test",
                "editor": { "vimMode": true },
                "modelProvider": {
                    "name": "Factory",
                    "baseUrl": "https://api.factory.ai/v1",
                    "envKey": "FACTORY_API_KEY",
                    "wireApi": "chat"
                },
                "rawJson": "{\"extra\":\"value\"}",
                "modelRotation": [
                    "claude-sonnet-4-5-20250929",
                    "gpt-5-codex"
                ],
                "listToolsOnStart": true
            })
        },
        "temperature": 0.5,
        "max_output_tokens": 64000,
        "approval_policy": "never",
        "sandbox_mode": "danger-full-access",
        "project_doc_max_bytes": 65536,
        "reasoning_effort": "high",
        "editor_vim_mode": true,
        "tools": {
            "url": "http://tools.test",
            "tools": [
                "memory_create_entities",
                "brave_search_brave_web_search"
            ]
        },
        "client_config": {
            "remoteTools": [
                "memory_create_entities",
                "brave_search_brave_web_search"
            ],
            "localServers": json!({})
        }
    });

    let factory_base_template_path = template_dir.join(CODE_FACTORY_CONTAINER_BASE_TEMPLATE);
    if factory_base_template_path.exists() {
        let base_template_content = std::fs::read_to_string(&factory_base_template_path)?;
        handlebars.register_partial("factory_container_base", base_template_content)?;
    } else {
        println!(
            "  ⚠️  Factory base container partial missing: {}",
            factory_base_template_path.display()
        );
    }

    for template_name in [
        CODE_FACTORY_CONTAINER_TEMPLATE,
        CODE_FACTORY_AGENTS_TEMPLATE,
    ] {
        let template_path = template_dir.join(template_name);

        if template_path.exists() {
            println!("  Testing {template_name}...");
            let template_content = std::fs::read_to_string(&template_path)?;
            handlebars.register_template_string(template_name, &template_content)?;
            let result = handlebars.render(template_name, &factory_data)?;
            println!("    ✅ Rendered successfully ({} chars)", result.len());
            for line in result.lines().take(3) {
                println!("    │ {line}");
            }
            if result.lines().count() > 3 {
                println!("    │ ... ({} total lines)", result.lines().count());
            }
            println!();
        } else {
            println!("  ⚠️  Template not found: {}", template_path.display());
        }
    }

    Ok(())
}
