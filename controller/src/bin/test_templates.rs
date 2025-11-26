#!/usr/bin/env cargo
//! Template testing utility for local handlebars template validation
//!
//! Usage: cargo run --bin `test_templates`

#![allow(clippy::disallowed_macros)]

use controller::tasks::template_paths::{
    CODE_CLAUDE_CONTAINER_TEMPLATE, CODE_CLAUDE_MEMORY_TEMPLATE, CODE_CLAUDE_SETTINGS_TEMPLATE,
    CODE_CODEX_AGENTS_TEMPLATE, CODE_CODEX_CONFIG_TEMPLATE, CODE_CODEX_CONTAINER_BASE_TEMPLATE,
    CODE_CODEX_CONTAINER_TEMPLATE, CODE_CURSOR_AGENTS_TEMPLATE,
    CODE_CURSOR_CONTAINER_BASE_TEMPLATE, CODE_CURSOR_CONTAINER_TEMPLATE,
    CODE_CURSOR_GLOBAL_CONFIG_TEMPLATE, CODE_CURSOR_PROJECT_CONFIG_TEMPLATE,
    CODE_FACTORY_AGENTS_TEMPLATE, CODE_FACTORY_CONTAINER_BASE_TEMPLATE,
    CODE_FACTORY_CONTAINER_TEMPLATE, CODE_FACTORY_GLOBAL_CONFIG_TEMPLATE,
    CODE_FACTORY_PROJECT_CONFIG_TEMPLATE, DOCS_CLAUDE_CLIENT_CONFIG_TEMPLATE,
    DOCS_CLAUDE_CONTAINER_TEMPLATE, DOCS_CLAUDE_MEMORY_TEMPLATE, DOCS_CLAUDE_PROMPT_TEMPLATE,
    DOCS_CLAUDE_SETTINGS_TEMPLATE, DOCS_CLAUDE_TOOLS_TEMPLATE,
};
use handlebars::Handlebars;
use serde_json::json;
use std::path::Path;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🧪 Testing Handlebars Templates...\n");

    // Initialize handlebars engine
    let mut handlebars = Handlebars::new();

    // Template directory - relative path from controller directory to infra/charts/controller/agent-templates
    let template_dir = Path::new("../infra/charts/controller/agent-templates");

    // Test docs templates
    test_docs_templates(&mut handlebars, template_dir)?;

    // Test code templates
    test_code_templates(&mut handlebars, template_dir)?;

    println!("✅ All templates rendered successfully!");
    Ok(())
}

fn test_docs_templates(
    handlebars: &mut Handlebars,
    template_dir: &Path,
) -> Result<(), Box<dyn std::error::Error>> {
    println!("📄 Testing Docs Templates:");

    // Mock DocsRunSpec data
    let docs_data = json!({
        "repository_url": "https://github.com/5dlabs/cto",
        "working_directory": "_projects/simple-api",
        "source_branch": "feature/example-project-and-cli",
        "model": "claude-3-5-sonnet-20241022",
        "github_user": "pm0-5dlabs",
        "remote_tools": ["rustdocs_query_rust_docs"],
        "tools_catalog": {
            "local": {
                "filesystem": {
                    "description": "Workspace filesystem access",
                    "command": "npx",
                    "args": ["-y", "@modelcontextprotocol/server-filesystem", "/workspace"],
                    "working_directory": "/workspace",
                    "tools": [
                        {
                            "name": "read_file",
                            "category": "filesystem",
                            "description": "Read file contents",
                            "use_cases": ["Inspect existing code" ]
                        }
                    ]
                }
            },
            "remote": {
                "docs": {
                    "description": "Documentation retrieval",
                    "endpoint": "http://tools/docs",
                    "tools": [
                        {
                            "name": "rustdocs_query_rust_docs",
                            "category": "documentation",
                            "description": "Search Rust documentation",
                            "use_cases": ["API lookups", "Trait discovery"]
                        }
                    ]
                }
            }
        },
        "total_tool_count": 2,
        "generated_timestamp": "2025-01-01T00:00:00Z"
    });

    // Test docs templates
    let docs_templates = [
        DOCS_CLAUDE_MEMORY_TEMPLATE,
        DOCS_CLAUDE_SETTINGS_TEMPLATE,
        DOCS_CLAUDE_CONTAINER_TEMPLATE,
        DOCS_CLAUDE_PROMPT_TEMPLATE,
        DOCS_CLAUDE_CLIENT_CONFIG_TEMPLATE,
        DOCS_CLAUDE_TOOLS_TEMPLATE,
    ];

    for template_name in &docs_templates {
        let template_path = template_dir.join(template_name);

        if template_path.exists() {
            println!("  Testing {template_name}...");

            // Register template
            let template_content = std::fs::read_to_string(&template_path)?;
            handlebars.register_template_string(template_name, &template_content)?;

            // Render template
            let result = handlebars.render(template_name, &docs_data)?;

            println!("    ✅ Rendered successfully ({} chars)", result.len());

            // Show first few lines of output for verification
            let lines: Vec<&str> = result.lines().take(3).collect();
            for line in lines {
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
        "model": "claude-3-5-sonnet-20241022",
        "github_user": "pm0-5dlabs",
        "local_tools": "bash,edit,read",
        "remote_tools": "github_create_issue",
        "tool_config": "default",
        "context_version": 1,
        "prompt_modification": null,
        "prompt_mode": "append"
    });

    for template_name in [
        CODE_CLAUDE_MEMORY_TEMPLATE,
        CODE_CLAUDE_SETTINGS_TEMPLATE,
        CODE_CLAUDE_CONTAINER_TEMPLATE,
    ] {
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

    for template_name in [
        CODE_CODEX_CONTAINER_TEMPLATE,
        CODE_CODEX_AGENTS_TEMPLATE,
        CODE_CODEX_CONFIG_TEMPLATE,
    ] {
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
                "rustdocs_query_rust_docs"
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
                "rustdocs_query_rust_docs"
            ]
        },
        "raw_additional_json": "{}",
        "client_config": {
            "remoteTools": [
                "memory_create_entities",
                "rustdocs_query_rust_docs"
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

    for template_name in [
        CODE_CURSOR_CONTAINER_TEMPLATE,
        CODE_CURSOR_AGENTS_TEMPLATE,
        CODE_CURSOR_GLOBAL_CONFIG_TEMPLATE,
        CODE_CURSOR_PROJECT_CONFIG_TEMPLATE,
    ] {
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
                "rustdocs_query_rust_docs"
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
                "rustdocs_query_rust_docs"
            ]
        },
        "client_config": {
            "remoteTools": [
                "memory_create_entities",
                "rustdocs_query_rust_docs"
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
        CODE_FACTORY_GLOBAL_CONFIG_TEMPLATE,
        CODE_FACTORY_PROJECT_CONFIG_TEMPLATE,
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
