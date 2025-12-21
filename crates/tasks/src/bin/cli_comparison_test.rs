//! CLI Comparison Test
//!
//! Tests PRD parsing across all supported CLI backends to establish
//! baseline quality and performance metrics.
//!
//! Results are saved to `tests/cli-comparison/` directory.

// Allow pedantic lints in test binary for cleaner test code
#![allow(
    clippy::needless_raw_string_hashes,
    clippy::const_is_empty,
    clippy::cast_precision_loss,
    clippy::match_same_arms,
    clippy::if_not_else,
    clippy::uninlined_format_args,
    clippy::ptr_arg,
    clippy::format_push_string,
    clippy::single_char_add_str,
    clippy::map_unwrap_or,
    clippy::disallowed_macros,
    clippy::redundant_closure_for_method_calls,
    clippy::too_many_lines,
    clippy::manual_string_new
)]

use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::{Duration, Instant};

use cli::CLIType;
use colored::Colorize;
use tasks::ai::cli_provider::CLITextGenerator;
use tasks::ai::ProviderRegistry;
use tasks::domain::AIDomain;
use tasks::entities::Task;
use tasks::storage::{FileStorage, Storage};

/// Output directory for test results
const OUTPUT_DIR: &str = "tests/cli-comparison";

/// Reference PRD for comparison testing
const REFERENCE_PRD: &str = r#"# Task Manager API

## Overview
Build a REST API for task management with authentication.

## Features

### Authentication
- JWT-based authentication
- Login/logout endpoints
- Token refresh capability

### Task Management
- CRUD operations for tasks
- Task status: pending, in-progress, done
- Task priority: low, medium, high

## Tech Stack
- Rust with Axum framework
- PostgreSQL database
- JWT for authentication
"#;

/// Expected themes that should appear in generated tasks
const EXPECTED_THEMES: &[&str] = &[
    "project",  // Project setup/initialization
    "database", // Database setup/migration
    "auth",     // Authentication system
    "jwt",      // JWT implementation
    "task",     // Task CRUD operations
    "api",      // API endpoints
];

/// Result of a single CLI test run
#[derive(Debug)]
struct CliTestResult {
    cli_type: CLIType,
    success: bool,
    duration: Duration,
    task_count: usize,
    tasks: Vec<Task>,
    error: Option<String>,
    themes_covered: Vec<String>,
    model_used: String,
}

impl CliTestResult {
    fn theme_coverage(&self) -> f64 {
        if EXPECTED_THEMES.is_empty() {
            return 0.0;
        }
        let covered = self
            .themes_covered
            .iter()
            .filter(|t| EXPECTED_THEMES.contains(&t.as_str()))
            .count();
        (covered as f64 / EXPECTED_THEMES.len() as f64) * 100.0
    }
}

/// Configuration for each CLI test
struct CliTestConfig {
    cli_type: CLIType,
    model: String,
    available: bool,
    extended_thinking: bool,
}

/// Check if CLI should use extended thinking
fn should_use_extended_thinking(cli_type: CLIType) -> bool {
    match cli_type {
        // Claude with Opus 4.5 supports thinking
        CLIType::Claude => true,
        // Cursor with Sonnet 4.5 supports thinking
        CLIType::Cursor => true,
        // OpenHands with Opus supports thinking
        CLIType::OpenHands => true,
        // Factory with Claude Opus 4.5 supports reasoning via --reasoning-effort
        CLIType::Factory => true,
        // Codex/OpenCode with o3 has built-in reasoning
        CLIType::Codex | CLIType::OpenCode => false,
        // Others don't support extended thinking
        _ => false,
    }
}

/// Get the latest model for each CLI type
fn get_default_model(cli_type: CLIType) -> &'static str {
    match cli_type {
        // Claude: Use Opus 4.5 with extended thinking
        CLIType::Claude => "claude-opus-4-5-20251101",
        // Codex: Use gpt-5.1-codex (latest ChatGPT Codex model)
        CLIType::Codex => "gpt-5.1-codex",
        // OpenCode: Use Claude Opus 4.5 via Anthropic (provider/model format)
        CLIType::OpenCode => "anthropic/claude-opus-4-5",
        // Cursor: Use opus-4.5-thinking (Cursor's supported model with thinking)
        CLIType::Cursor => "opus-4.5-thinking",
        // Factory: Use Claude Opus 4.5 via droid (their default)
        CLIType::Factory => "claude-opus-4-5-20251101",
        // OpenHands: Default model
        CLIType::OpenHands => "claude-opus-4-5-20251101",
        // Grok: Use Grok 3
        CLIType::Grok => "grok-3",
        // Gemini: Use Gemini 2.5 Flash (better quota availability)
        CLIType::Gemini => "gemini-2.5-flash",
        // Qwen: Use Qwen Max
        CLIType::Qwen => "qwen-max",
        // Dexter: Use Claude Sonnet 4
        CLIType::Dexter => "claude-sonnet-4-20250514",
    }
}

/// Check if a CLI is available on the system
fn check_cli_available(cli_type: CLIType) -> bool {
    let executable = match cli_type {
        CLIType::Claude => "claude",
        CLIType::Codex => "codex",
        CLIType::Cursor => "cursor",
        CLIType::Factory => "droid",
        CLIType::OpenCode => "opencode",
        CLIType::Gemini => "gemini",
        CLIType::Grok => "grok",
        CLIType::OpenHands => "openhands",
        CLIType::Qwen => "qwen",
        CLIType::Dexter => "dexter-agent",
    };

    // Check PATH
    if which::which(executable).is_ok() {
        return true;
    }

    // Check common installation locations for Claude CLI
    if matches!(cli_type, CLIType::Claude) {
        if let Ok(home) = std::env::var("HOME") {
            let claude_path = format!("{}/.claude/local/claude", home);
            if std::path::Path::new(&claude_path).exists() {
                return true;
            }
        }
    }

    false
}

/// Extract themes from generated tasks
fn extract_themes(tasks: &[Task]) -> Vec<String> {
    let mut themes = Vec::new();
    let keywords: HashMap<&str, &str> = [
        ("project", "project|setup|init|scaffold"),
        ("database", "database|postgres|migration|schema|sql"),
        ("auth", "auth|login|logout|register"),
        ("jwt", "jwt|token|bearer"),
        ("task", "task|crud|create|read|update|delete"),
        ("api", "api|endpoint|route|handler"),
        ("error", "error|exception|handling"),
        ("test", "test|testing|spec"),
        ("docker", "docker|container|deploy"),
    ]
    .into_iter()
    .collect();

    for (theme, pattern) in keywords {
        let pattern_lower = pattern.to_lowercase();
        let patterns: Vec<&str> = pattern_lower.split('|').collect();

        for task in tasks {
            let combined = format!(
                "{} {} {}",
                task.title.to_lowercase(),
                task.description.to_lowercase(),
                task.details.to_lowercase()
            );

            if patterns.iter().any(|p| combined.contains(p)) {
                if !themes.contains(&theme.to_string()) {
                    themes.push(theme.to_string());
                }
                break;
            }
        }
    }

    themes
}

/// Run a single CLI test
async fn run_cli_test(config: &CliTestConfig, storage: Arc<dyn Storage>) -> CliTestResult {
    let start = Instant::now();

    if !config.available {
        return CliTestResult {
            cli_type: config.cli_type,
            success: false,
            duration: Duration::ZERO,
            task_count: 0,
            tasks: vec![],
            error: Some("CLI not available".to_string()),
            themes_covered: vec![],
            model_used: config.model.clone(),
        };
    }

    // Create provider for benchmarking (no MCP config to avoid tool explanations)
    let provider = CLITextGenerator::for_benchmark(config.cli_type, config.extended_thinking);

    let provider = match provider {
        Ok(p) => p,
        Err(e) => {
            return CliTestResult {
                cli_type: config.cli_type,
                success: false,
                duration: start.elapsed(),
                task_count: 0,
                tasks: vec![],
                error: Some(format!("Failed to create provider: {e}")),
                themes_covered: vec![],
                model_used: config.model.clone(),
            };
        }
    };

    let registry = ProviderRegistry::new();
    registry.register(Arc::new(provider));
    let ai_domain = AIDomain::with_registry(storage, registry);

    // Run PRD parsing with research mode enabled
    match ai_domain
        .parse_prd(
            REFERENCE_PRD,
            "reference-prd.md",
            Some(5),
            true,
            Some(&config.model),
        )
        .await
    {
        Ok((tasks, _usage)) => {
            let themes = extract_themes(&tasks);
            CliTestResult {
                cli_type: config.cli_type,
                success: true,
                duration: start.elapsed(),
                task_count: tasks.len(),
                tasks,
                error: None,
                themes_covered: themes,
                model_used: config.model.clone(),
            }
        }
        Err(e) => CliTestResult {
            cli_type: config.cli_type,
            success: false,
            duration: start.elapsed(),
            task_count: 0,
            tasks: vec![],
            error: Some(format!("{e}")),
            themes_covered: vec![],
            model_used: config.model.clone(),
        },
    }
}

/// Save test results to output directory
fn save_cli_output(result: &CliTestResult, output_dir: &PathBuf) -> std::io::Result<()> {
    let cli_name = format!("{}", result.cli_type);
    let cli_dir = output_dir.join(&cli_name);
    std::fs::create_dir_all(&cli_dir)?;

    // Save tasks as JSON
    let tasks_json = serde_json::json!({
        "cli": cli_name,
        "model": result.model_used,
        "success": result.success,
        "duration_ms": result.duration.as_millis(),
        "task_count": result.task_count,
        "theme_coverage": result.theme_coverage(),
        "themes_covered": result.themes_covered,
        "error": result.error,
        "tasks": result.tasks.iter().map(|t| serde_json::json!({
            "id": t.id,
            "title": t.title,
            "status": format!("{}", t.status),
            "priority": format!("{}", t.priority),
            "description": t.description,
            "details": t.details,
            "test_strategy": t.test_strategy,
            "dependencies": t.dependencies,
        })).collect::<Vec<_>>()
    });
    std::fs::write(
        cli_dir.join("tasks.json"),
        serde_json::to_string_pretty(&tasks_json)?,
    )?;

    // Save tasks as Markdown document
    let mut md = String::new();
    md.push_str(&format!(
        "# {} Task Generation Results\n\n",
        cli_name.to_uppercase()
    ));
    md.push_str(&format!("**Model:** {}\n", result.model_used));
    md.push_str(&format!(
        "**Duration:** {:.2}s\n",
        result.duration.as_secs_f64()
    ));
    md.push_str(&format!("**Tasks Generated:** {}\n", result.task_count));
    md.push_str(&format!(
        "**Theme Coverage:** {:.0}%\n",
        result.theme_coverage()
    ));
    md.push_str(&format!(
        "**Themes Covered:** {}\n\n",
        result.themes_covered.join(", ")
    ));

    if let Some(ref err) = result.error {
        md.push_str(&format!("**Error:** {}\n\n", err));
    }

    md.push_str("---\n\n");

    for task in &result.tasks {
        md.push_str(&format!("## Task {}: {}\n\n", task.id, task.title));
        md.push_str(&format!(
            "**Status:** {} | **Priority:** {}\n\n",
            task.status, task.priority
        ));

        if !task.dependencies.is_empty() {
            md.push_str(&format!(
                "**Dependencies:** {}\n\n",
                task.dependencies.join(", ")
            ));
        }

        md.push_str("### Description\n\n");
        md.push_str(&task.description);
        md.push_str("\n\n");

        md.push_str("### Implementation Details\n\n");
        md.push_str(&task.details);
        md.push_str("\n\n");

        md.push_str("### Test Strategy\n\n");
        md.push_str(&task.test_strategy);
        md.push_str("\n\n---\n\n");
    }

    std::fs::write(cli_dir.join("tasks.md"), md)?;

    Ok(())
}

/// Print a summary table of all results
fn print_results_summary(results: &[CliTestResult]) {
    println!("\n{}", "═".repeat(100));
    println!("{}", "CLI COMPARISON TEST RESULTS".bold().cyan());
    println!("{}", "═".repeat(100));

    // Header
    println!(
        "{:<12} {:<8} {:<10} {:<8} {:<12} {:<30} {}",
        "CLI".bold(),
        "Status".bold(),
        "Duration".bold(),
        "Tasks".bold(),
        "Coverage".bold(),
        "Model".bold(),
        "Themes".bold()
    );
    println!("{}", "─".repeat(100));

    // Results
    for result in results {
        let status = if result.success {
            "✓".green().to_string()
        } else {
            "✗".red().to_string()
        };

        let duration = if result.duration.as_secs() > 0 {
            format!("{:.1}s", result.duration.as_secs_f64())
        } else {
            format!("{}ms", result.duration.as_millis())
        };

        let coverage = format!("{:.0}%", result.theme_coverage());
        let themes = result.themes_covered.join(", ");

        println!(
            "{:<12} {:<8} {:<10} {:<8} {:<12} {:<30} {}",
            format!("{}", result.cli_type),
            status,
            duration,
            result.task_count,
            coverage,
            &result.model_used[..result.model_used.len().min(28)],
            themes
        );

        if let Some(ref err) = result.error {
            println!("             {}", format!("Error: {}", err).red());
        }
    }

    println!("{}", "═".repeat(100));

    // Summary stats
    let successful = results.iter().filter(|r| r.success).count();
    let total = results.len();
    let avg_tasks: f64 = results
        .iter()
        .filter(|r| r.success)
        .map(|r| r.task_count as f64)
        .sum::<f64>()
        / successful.max(1) as f64;
    let avg_coverage: f64 = results
        .iter()
        .filter(|r| r.success)
        .map(|r| r.theme_coverage())
        .sum::<f64>()
        / successful.max(1) as f64;

    println!("\n{}", "Summary".bold());
    println!("  Successful: {}/{}", successful, total);
    println!("  Avg Tasks: {:.1}", avg_tasks);
    println!("  Avg Theme Coverage: {:.0}%", avg_coverage);
}

/// Print detailed task comparison
fn print_task_details(results: &[CliTestResult]) {
    println!("\n{}", "═".repeat(100));
    println!("{}", "DETAILED TASK COMPARISON".bold().cyan());
    println!("{}", "═".repeat(100));

    for result in results.iter().filter(|r| r.success) {
        println!(
            "\n{} ({})",
            format!("{}", result.cli_type).bold(),
            result.model_used
        );
        println!("{}", "─".repeat(60));

        for task in &result.tasks {
            let priority_color = match task.priority.to_string().as_str() {
                "high" => task.priority.to_string().red(),
                "medium" => task.priority.to_string().yellow(),
                _ => task.priority.to_string().green(),
            };

            println!(
                "  {} [{}] {}",
                format!("Task {}:", task.id).bold(),
                priority_color,
                task.title
            );

            // Show first 100 chars of description
            let desc_preview = if task.description.len() > 100 {
                format!("{}...", &task.description[..100])
            } else {
                task.description.clone()
            };
            println!("     {}", desc_preview.dimmed());
        }
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logging
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::from_default_env()
                .add_directive("tasks=info".parse().unwrap()),
        )
        .init();

    println!("{}", "CLI Comparison Test".bold().cyan());
    println!("Testing PRD parsing across all available CLI backends\n");

    // Create temporary storage
    let temp_dir = tempfile::tempdir()?;
    let storage = Arc::new(FileStorage::new(temp_dir.path()));

    // Initialize storage
    storage.initialize().await?;

    // Define all CLIs to test
    // Note: Cursor is skipped because `cursor` is the IDE, not an AI agent CLI
    let all_cli_types = [
        CLIType::Claude,
        CLIType::Codex,
        CLIType::OpenCode,
        CLIType::Cursor,
        CLIType::Factory,
        CLIType::Gemini,
        // Note: OpenHands, Grok, and Qwen are not currently used/installed
    ];

    // Check availability and create configs
    let configs: Vec<CliTestConfig> = all_cli_types
        .iter()
        .map(|&cli_type| {
            let available = check_cli_available(cli_type);
            let extended_thinking = should_use_extended_thinking(cli_type);
            let model = get_default_model(cli_type);
            println!(
                "  {} {:12} {:30} {}",
                if available {
                    "✓".green()
                } else {
                    "✗".red()
                },
                format!("{}", cli_type),
                model,
                if extended_thinking {
                    "+thinking".cyan().to_string()
                } else {
                    "".to_string()
                }
            );
            CliTestConfig {
                cli_type,
                model: model.to_string(),
                available,
                extended_thinking,
            }
        })
        .collect();

    let available_count = configs.iter().filter(|c| c.available).count();
    println!("\nFound {} available CLIs\n", available_count);

    if available_count == 0 {
        println!("{}", "No CLIs available for testing!".red());
        return Ok(());
    }

    // Create output directory
    let output_dir = PathBuf::from(OUTPUT_DIR);
    std::fs::create_dir_all(&output_dir)?;

    // Save reference PRD
    std::fs::write(output_dir.join("reference-prd.md"), REFERENCE_PRD)?;

    // Run tests sequentially (to avoid rate limiting)
    let mut results = Vec::new();
    for config in &configs {
        if config.available {
            let thinking_str = if config.extended_thinking {
                " (+thinking)"
            } else {
                ""
            };
            println!(
                "\n{} Testing {} with {}{}...",
                "→".cyan(),
                format!("{}", config.cli_type).bold(),
                config.model,
                thinking_str.cyan()
            );
        }
        let result = run_cli_test(config, Arc::clone(&storage) as Arc<dyn Storage>).await;

        // Save individual CLI output
        if result.success {
            if let Err(e) = save_cli_output(&result, &output_dir) {
                eprintln!("  {} Failed to save output: {}", "⚠".yellow(), e);
            } else {
                println!(
                    "  {} Saved {} tasks to {}/{}/",
                    "✓".green(),
                    result.task_count,
                    OUTPUT_DIR,
                    result.cli_type
                );
            }
        }

        results.push(result);
    }

    // Print results
    print_results_summary(&results);
    print_task_details(&results);

    // Save aggregate results to JSON
    let results_json = serde_json::json!({
        "timestamp": chrono::Utc::now().to_rfc3339(),
        "prd": REFERENCE_PRD,
        "expected_themes": EXPECTED_THEMES,
        "results": results.iter().map(|r| serde_json::json!({
            "cli": format!("{}", r.cli_type),
            "model": r.model_used,
            "success": r.success,
            "extended_thinking": configs.iter()
                .find(|c| c.cli_type == r.cli_type)
                .map(|c| c.extended_thinking)
                .unwrap_or(false),
            "duration_ms": r.duration.as_millis(),
            "task_count": r.task_count,
            "theme_coverage": r.theme_coverage(),
            "themes_covered": r.themes_covered,
            "error": r.error,
            "tasks": r.tasks.iter().map(|t| serde_json::json!({
                "id": t.id,
                "title": t.title,
                "priority": format!("{}", t.priority),
                "description": t.description,
            })).collect::<Vec<_>>()
        })).collect::<Vec<_>>()
    });

    let results_path = output_dir.join("summary.json");
    std::fs::write(&results_path, serde_json::to_string_pretty(&results_json)?)?;
    println!(
        "\n{} Aggregate results saved to {}",
        "✓".green(),
        results_path.display()
    );

    // Generate comparison report
    generate_comparison_report(&results, &output_dir)?;

    Ok(())
}

/// Generate a markdown comparison report
fn generate_comparison_report(
    results: &[CliTestResult],
    output_dir: &PathBuf,
) -> std::io::Result<()> {
    let mut md = String::new();

    md.push_str("# CLI Comparison Test Report\n\n");
    md.push_str(&format!(
        "**Generated:** {}\n\n",
        chrono::Utc::now().format("%Y-%m-%d %H:%M:%S UTC")
    ));

    // Summary table
    md.push_str("## Summary\n\n");
    md.push_str("| CLI | Model | Status | Duration | Tasks | Coverage | Themes |\n");
    md.push_str("|-----|-------|--------|----------|-------|----------|--------|\n");

    for result in results {
        let status = if result.success { "✓" } else { "✗" };
        let duration = format!("{:.1}s", result.duration.as_secs_f64());
        let coverage = format!("{:.0}%", result.theme_coverage());

        md.push_str(&format!(
            "| {} | {} | {} | {} | {} | {} | {} |\n",
            result.cli_type,
            result.model_used,
            status,
            duration,
            result.task_count,
            coverage,
            result.themes_covered.join(", ")
        ));
    }

    // Errors section
    let errors: Vec<_> = results.iter().filter(|r| r.error.is_some()).collect();
    if !errors.is_empty() {
        md.push_str("\n## Errors\n\n");
        for result in errors {
            md.push_str(&format!(
                "### {}\n\n```\n{}\n```\n\n",
                result.cli_type,
                result.error.as_ref().unwrap()
            ));
        }
    }

    // Task comparison
    md.push_str("\n## Task Titles by CLI\n\n");
    for result in results.iter().filter(|r| r.success) {
        md.push_str(&format!("### {}\n\n", result.cli_type));
        for task in &result.tasks {
            md.push_str(&format!(
                "- **Task {}** [{}]: {}\n",
                task.id, task.priority, task.title
            ));
        }
        md.push_str("\n");
    }

    // Links to detailed outputs
    md.push_str("## Detailed Outputs\n\n");
    for result in results.iter().filter(|r| r.success) {
        md.push_str(&format!(
            "- [{}](./{}/tasks.md) | [JSON](./{}/tasks.json)\n",
            result.cli_type, result.cli_type, result.cli_type
        ));
    }

    std::fs::write(output_dir.join("REPORT.md"), md)?;
    println!(
        "{} Comparison report saved to {}/REPORT.md",
        "✓".green(),
        output_dir.display()
    );

    Ok(())
}
