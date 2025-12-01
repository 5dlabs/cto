//! CLI for utils micro-utilities
//!
//! Run `utils --help` for usage information.

// CLI binaries legitimately need println! for user output
#![allow(clippy::disallowed_macros)]

use anyhow::Result;
use clap::{Parser, Subcommand};
use tracing_subscriber::{fmt, prelude::*, EnvFilter};
use utils::{AnnotationLevel, PrAlerts, PrComment};

#[derive(Parser)]
#[command(name = "utils")]
#[command(about = "Micro utilities for GitHub operations")]
#[command(version)]
struct Cli {
    /// Output format: json, text
    #[arg(short, long, default_value = "text")]
    format: OutputFormat,

    /// Enable verbose logging
    #[arg(short, long)]
    verbose: bool,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Clone, Copy, Default, clap::ValueEnum)]
enum OutputFormat {
    #[default]
    Text,
    Json,
}

#[derive(Subcommand)]
enum Commands {
    /// Fetch PR check run annotations (alerts)
    Alerts {
        /// Repository in owner/repo format
        #[arg(short, long)]
        repo: String,

        /// Pull request number
        #[arg(short, long)]
        pr: u32,

        /// Filter by level: notice, warning, failure
        #[arg(short, long)]
        level: Option<String>,

        /// Only show summary (check run counts, no details)
        #[arg(short, long)]
        summary: bool,
    },

    /// Fetch failure alerts and post them as a PR comment
    PostAlerts {
        /// Repository in owner/repo format
        #[arg(short, long)]
        repo: String,

        /// Pull request number
        #[arg(short, long)]
        pr: u32,

        /// Include warnings in addition to failures
        #[arg(short, long)]
        warnings: bool,

        /// Dry run - print comment instead of posting
        #[arg(short, long)]
        dry_run: bool,
    },
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    // Initialize tracing
    let filter = if cli.verbose {
        EnvFilter::new("debug")
    } else {
        EnvFilter::new("warn")
    };

    tracing_subscriber::registry()
        .with(fmt::layer().with_target(false))
        .with(filter)
        .init();

    match cli.command {
        Commands::Alerts {
            repo,
            pr,
            level,
            summary,
        } => {
            run_alerts(&repo, pr, level, summary, cli.format).await?;
        }
        Commands::PostAlerts {
            repo,
            pr,
            warnings,
            dry_run,
        } => {
            run_post_alerts(&repo, pr, warnings, dry_run).await?;
        }
    }

    Ok(())
}

async fn run_post_alerts(repo: &str, pr: u32, include_warnings: bool, dry_run: bool) -> Result<()> {
    let (owner, repo_name) = utils::alerts::parse_repo(repo)?;
    let alerts_client = PrAlerts::new(owner, repo_name);

    // Get head SHA for fetching file contents
    let head_sha = alerts_client.get_head_sha(pr).await?;

    // Fetch failures (and optionally warnings)
    let mut annotations = alerts_client.fetch_failures(pr).await?;

    if include_warnings {
        let warnings = alerts_client.fetch_warnings(pr).await?;
        annotations.extend(warnings);
    }

    // Create comment client for posting and fetching file context
    let comment_client = PrComment::new(owner, repo_name);

    if dry_run {
        // For dry run, format with context but don't post
        println!("=== Dry run: would post this comment to PR #{pr} ===\n");
        println!("Fetching file contents for code snippets...\n");
        let comment_body = comment_client
            .format_alerts_with_context(&head_sha, &annotations)
            .await;
        println!("{comment_body}");
        return Ok(());
    }

    // Post the comment with full context
    comment_client
        .post_alerts_with_context(pr, &head_sha, &annotations)
        .await?;

    println!("✅ Posted alerts summary to PR #{pr}");
    if annotations.is_empty() {
        println!("   (No failures found - posted success message)");
    } else {
        println!(
            "   ({} alert(s) included with code snippets)",
            annotations.len()
        );
    }

    Ok(())
}

async fn run_alerts(
    repo: &str,
    pr: u32,
    level: Option<String>,
    summary: bool,
    format: OutputFormat,
) -> Result<()> {
    let (owner, repo_name) = utils::alerts::parse_repo(repo)?;
    let client = PrAlerts::new(owner, repo_name);

    if summary {
        let runs = client.get_summary(pr).await?;
        match format {
            OutputFormat::Json => {
                println!("{}", serde_json::to_string_pretty(&runs)?);
            }
            OutputFormat::Text => {
                if runs.is_empty() {
                    println!("No check runs with annotations for PR #{pr}");
                } else {
                    println!("Check runs with annotations for PR #{pr}:");
                    for run in runs {
                        println!("  {} ({}): {} annotations", run.name, run.id, run.count);
                    }
                }
            }
        }
        return Ok(());
    }

    let annotations = match level.as_deref() {
        Some("failure") => client.fetch_by_level(pr, AnnotationLevel::Failure).await?,
        Some("warning") => client.fetch_by_level(pr, AnnotationLevel::Warning).await?,
        Some("notice") => client.fetch_by_level(pr, AnnotationLevel::Notice).await?,
        Some(other) => anyhow::bail!("Invalid level '{other}'. Use: notice, warning, failure"),
        None => client.fetch(pr).await?,
    };

    match format {
        OutputFormat::Json => {
            println!("{}", serde_json::to_string_pretty(&annotations)?);
        }
        OutputFormat::Text => {
            if annotations.is_empty() {
                println!("No annotations found for PR #{pr}");
            } else {
                println!("Annotations for PR #{pr} ({} total):\n", annotations.len());
                for ann in annotations {
                    println!(
                        "[{}] {}:{}",
                        ann.level.to_string().to_uppercase(),
                        ann.path,
                        ann.start_line
                    );
                    println!("  {}", ann.message);
                    if !ann.title.is_empty() {
                        println!("  Title: {}", ann.title);
                    }
                    println!();
                }
            }
        }
    }

    Ok(())
}
