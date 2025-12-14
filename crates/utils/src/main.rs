//! CLI for utils micro-utilities
//!
//! Run `utils --help` for usage information.

// CLI binaries legitimately need println! for user output
#![allow(clippy::disallowed_macros)]

use anyhow::Result;
use clap::{Parser, Subcommand};
use tracing_subscriber::{fmt, prelude::*, EnvFilter};
use utils::{
    AnnotationLevel, CiCheck, ClippyErrors, PrAlerts, PrCheckStatus, PrChecks, PrComment,
    PrConversations, PrReviews,
};

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

    /// Fetch PR review comments from Bugbot and Stitch
    Reviews {
        /// Repository in owner/repo format
        #[arg(short, long)]
        repo: String,

        /// Pull request number
        #[arg(short, long)]
        pr: u32,

        /// Filter by author: bugbot, stitch, or username
        #[arg(short, long)]
        author: Option<String>,

        /// Only show inline comments (with `file:line`)
        #[arg(short, long)]
        inline: bool,
    },

    /// Fetch Clippy errors from failed lint-rust CI check
    Clippy {
        /// Repository in owner/repo format
        #[arg(short, long)]
        repo: String,

        /// Pull request number
        #[arg(short, long)]
        pr: u32,

        /// Output as fix prompt for AI remediation
        #[arg(long)]
        prompt: bool,

        /// Specific check run ID (optional, defaults to finding lint-rust)
        #[arg(long)]
        check_run_id: Option<u64>,
    },

    /// Resolve PR review thread conversations
    Resolve {
        /// Repository in owner/repo format
        #[arg(short, long)]
        repo: String,

        /// Pull request number
        #[arg(short, long)]
        pr: u32,

        /// Resolve all unresolved conversations
        #[arg(long)]
        all: bool,

        /// Only resolve conversations by this author
        #[arg(short, long)]
        author: Option<String>,

        /// Specific thread ID to resolve
        #[arg(long)]
        thread_id: Option<String>,

        /// List unresolved conversations without resolving
        #[arg(long)]
        list: bool,
    },
    /// Check PR CI status (pass/fail/pending)
    Checks {
        /// Repository in owner/repo format
        #[arg(short, long)]
        repo: String,

        /// Pull request number
        #[arg(short, long)]
        pr: u32,

        /// Only show failed checks
        #[arg(long)]
        failed: bool,

        /// Only show pending checks
        #[arg(long)]
        pending: bool,

        /// Wait for checks to complete (timeout in seconds)
        #[arg(long)]
        wait: Option<u64>,

        /// Poll interval in seconds when waiting (default: 30)
        #[arg(long)]
        interval: Option<u64>,

        /// Exit with error code if any checks fail
        #[arg(long)]
        strict: bool,
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
        Commands::Reviews {
            repo,
            pr,
            author,
            inline,
        } => {
            run_reviews(&repo, pr, author, inline, cli.format).await?;
        }
        Commands::Clippy {
            repo,
            pr,
            prompt,
            check_run_id,
        } => {
            run_clippy(&repo, pr, prompt, check_run_id, cli.format).await?;
        }
        Commands::Resolve {
            repo,
            pr,
            all,
            author,
            thread_id,
            list,
        } => {
            run_resolve(&repo, pr, all, author, thread_id, list, cli.format).await?;
        }
        Commands::Checks {
            repo,
            pr,
            failed,
            pending,
            wait,
            interval,
            strict,
        } => {
            let exit_code = run_checks(
                &repo, pr, failed, pending, wait, interval, strict, cli.format,
            )
            .await?;
            if exit_code != 0 {
                std::process::exit(exit_code);
            }
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

async fn run_reviews(
    repo: &str,
    pr: u32,
    author: Option<String>,
    inline_only: bool,
    format: OutputFormat,
) -> Result<()> {
    let (owner, repo_name) = utils::alerts::parse_repo(repo)?;
    let client = PrReviews::new(owner, repo_name);

    let comments = match author.as_deref() {
        Some("bugbot" | "bug-bot" | "cursor") => client.fetch_bugbot(pr).await?,
        Some("stitch") => client.fetch_stitch(pr).await?,
        Some(author_name) => client.fetch_by_author(pr, author_name).await?,
        None => client.fetch(pr).await?, // Default: Bugbot + Stitch
    };

    // Filter to inline only if requested
    let comments: Vec<_> = if inline_only {
        comments
            .into_iter()
            .filter(|c| c.line.is_some() && !c.path.is_empty())
            .collect()
    } else {
        comments
    };

    match format {
        OutputFormat::Json => {
            println!("{}", serde_json::to_string_pretty(&comments)?);
        }
        OutputFormat::Text => {
            if comments.is_empty() {
                println!("No review comments found for PR #{pr}");
            } else {
                println!("Review comments for PR #{pr} ({} total):\n", comments.len());
                for comment in comments {
                    // Header with author and location
                    let location = if !comment.path.is_empty() && comment.line.is_some() {
                        format!("{}:{}", comment.path, comment.line.unwrap())
                    } else if !comment.path.is_empty() {
                        comment.path.clone()
                    } else {
                        "general comment".to_string()
                    };

                    let source = if comment.is_bugbot {
                        "🤖 Bugbot"
                    } else if comment.is_stitch {
                        "🧵 Stitch"
                    } else {
                        "👤"
                    };

                    println!("[{source}] @{} at {location}", comment.author);

                    // Truncate body for display
                    let body_preview: String = comment
                        .body
                        .lines()
                        .take(3)
                        .collect::<Vec<_>>()
                        .join("\n  ");
                    println!("  {body_preview}");

                    // Show suggestion if present
                    if let Some(suggestion) = &comment.suggestion {
                        println!(
                            "  💡 Suggestion: {}",
                            suggestion.lines().next().unwrap_or("")
                        );
                    }

                    println!();
                }
            }
        }
    }

    Ok(())
}

/// Run the clippy command to fetch and display Clippy errors
async fn run_clippy(
    repo: &str,
    pr: u32,
    as_prompt: bool,
    check_run_id: Option<u64>,
    format: OutputFormat,
) -> Result<()> {
    let (owner, repo_name) = utils::alerts::parse_repo(repo)?;
    let client = ClippyErrors::new(owner, repo_name);

    let errors = if let Some(id) = check_run_id {
        client.fetch_by_check_run(id).await?
    } else {
        client.fetch(pr).await?
    };

    if errors.is_empty() {
        println!("✅ No Clippy errors found for PR #{pr}");
        return Ok(());
    }

    if as_prompt {
        // Output as a fix prompt for AI remediation
        println!("{}", client.generate_fix_prompt(&errors));
    } else {
        match format {
            OutputFormat::Json => {
                println!("{}", serde_json::to_string_pretty(&errors)?);
            }
            OutputFormat::Text => {
                println!("🔴 Found {} Clippy errors for PR #{}:\n", errors.len(), pr);
                for (i, err) in errors.iter().enumerate() {
                    println!(
                        "{}. [{}] {} at {}:{}",
                        i + 1,
                        err.level,
                        err.code,
                        err.file,
                        err.line
                    );
                    println!("   Message: {}", err.message);
                    if let Some(suggestion) = &err.suggestion {
                        println!("   💡 {suggestion}");
                    }
                    println!();
                }
            }
        }
    }

    Ok(())
}

/// Run the resolve command to manage PR conversations
async fn run_resolve(
    repo: &str,
    pr: u32,
    all: bool,
    author: Option<String>,
    thread_id: Option<String>,
    list: bool,
    format: OutputFormat,
) -> Result<()> {
    let (owner, repo_name) = utils::alerts::parse_repo(repo)?;
    let client = PrConversations::new(owner, repo_name);

    // List mode - show unresolved conversations
    if list {
        let threads = client.list_unresolved(pr).await?;

        if threads.is_empty() {
            println!("✅ No unresolved conversations for PR #{pr}");
            return Ok(());
        }

        match format {
            OutputFormat::Json => {
                println!("{}", serde_json::to_string_pretty(&threads)?);
            }
            OutputFormat::Text => {
                println!(
                    "📝 {} unresolved conversations for PR #{}:\n",
                    threads.len(),
                    pr
                );
                for (i, thread) in threads.iter().enumerate() {
                    let location = thread.path.as_ref().map_or_else(
                        || "general".to_string(),
                        |p| {
                            thread
                                .line
                                .map_or_else(|| p.clone(), |l| format!("{p}:{l}"))
                        },
                    );

                    println!(
                        "{}. [{}] @{} at {}",
                        i + 1,
                        &thread.id[..12.min(thread.id.len())],
                        thread.author,
                        location
                    );
                    println!("   {}", thread.body_preview);
                    println!();
                }
            }
        }
        return Ok(());
    }

    // Resolve specific thread by ID
    if let Some(id) = thread_id {
        match client.resolve(&id).await {
            Ok(true) => println!("✅ Resolved thread {id}"),
            Ok(false) => println!("⚠️ Thread {id} is not a conversation (skipped)"),
            Err(e) => println!("❌ Failed to resolve thread {id}: {e}"),
        }
        return Ok(());
    }

    // Resolve by author
    if let Some(author) = author {
        let resolved = client.resolve_by_author(pr, &author).await?;
        println!("✅ Resolved {resolved} conversations by @{author} on PR #{pr}");
        return Ok(());
    }

    // Resolve all
    if all {
        let resolved = client.resolve_all(pr).await?;
        println!("✅ Resolved {resolved} conversations on PR #{pr}");
        return Ok(());
    }

    // Default: list unresolved
    let threads = client.list_unresolved(pr).await?;
    println!(
        "📝 {} unresolved conversations on PR #{}. Use --list, --all, --author, or --thread-id",
        threads.len(),
        pr
    );

    Ok(())
}

/// Run the checks command to fetch and display PR check status
#[allow(clippy::fn_params_excessive_bools, clippy::too_many_arguments)]
async fn run_checks(
    repo: &str,
    pr: u32,
    show_failed: bool,
    show_pending: bool,
    wait: Option<u64>,
    interval: Option<u64>,
    strict: bool,
    format: OutputFormat,
) -> Result<i32> {
    let (owner, repo_name) = utils::alerts::parse_repo(repo)?;
    let client = PrChecks::new(owner, repo_name);

    // If waiting, poll until complete
    let status = if let Some(timeout) = wait {
        client.wait_for_completion(pr, timeout, interval).await?
    } else {
        client.fetch(pr).await?
    };

    // Filter based on flags
    let checks_to_show: Vec<_> = if show_failed {
        status.failed_checks().into_iter().cloned().collect()
    } else if show_pending {
        status.pending_checks().into_iter().cloned().collect()
    } else {
        status.checks.clone()
    };

    // Output
    match format {
        OutputFormat::Json => {
            if show_failed || show_pending {
                println!("{}", serde_json::to_string_pretty(&checks_to_show)?);
            } else {
                println!("{}", serde_json::to_string_pretty(&status)?);
            }
        }
        OutputFormat::Text => {
            print_checks_text(pr, &status, &checks_to_show, show_failed, show_pending);
        }
    }

    // Determine exit code
    let exit_code = if strict && status.has_failures() {
        1
    } else if strict && status.has_pending() {
        2
    } else {
        0
    };

    Ok(exit_code)
}

/// Print check status in human-readable format
fn print_checks_text(
    pr: u32,
    status: &PrCheckStatus,
    checks: &[CiCheck],
    filtered_failed: bool,
    filtered_pending: bool,
) {
    // Header
    let filter_desc = if filtered_failed {
        " (failed only)"
    } else if filtered_pending {
        " (pending only)"
    } else {
        ""
    };

    println!("PR #{pr} Check Status{filter_desc}\n");

    // Summary line
    let status_emoji = if status.all_passed() {
        "✅"
    } else if status.has_failures() {
        "❌"
    } else if status.has_pending() {
        "⏳"
    } else {
        "ℹ️"
    };

    println!(
        "{status_emoji} {}/{} checks passed ({} failed, {} pending, {} skipped)",
        status.passed, status.total, status.failed, status.pending, status.skipped
    );
    println!("   Merge state: {}\n", status.merge_state);

    if checks.is_empty() {
        if filtered_failed {
            println!("No failed checks found.");
        } else if filtered_pending {
            println!("No pending checks found.");
        }
        return;
    }

    // List checks
    for check in checks {
        let icon = if check.passed() {
            "✅"
        } else if check.failed() {
            "❌"
        } else if check.pending() {
            "⏳"
        } else {
            "⚪"
        };

        let conclusion_str = check
            .conclusion
            .map_or_else(|| "pending".to_string(), |c| c.to_string());

        let workflow = if check.workflow.is_empty() {
            String::new()
        } else {
            format!(" ({})", check.workflow)
        };

        println!("{icon} {}{workflow}: {conclusion_str}", check.name);
    }

    // Instructions for failed checks
    if status.has_failures() && !filtered_pending {
        println!("\n💡 To see check run logs, use: gh run view <run-id>");
    }
}
