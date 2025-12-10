//! Research CLI - Twitter/X bookmark research pipeline.

#![allow(clippy::doc_markdown)] // CronJob etc. don't need backticks
#![allow(clippy::unused_async)] // Async signature needed for interface consistency

use anyhow::Result;
use clap::{Parser, Subcommand};
use std::path::{Path, PathBuf};
use tracing_subscriber::{fmt, prelude::*, EnvFilter};

use research::analysis::Category;
use research::auth::{BrowserAuth, Session};
use research::digest::{DigestAnalyzer, DigestConfig, DigestGenerator, DigestState, EmailSender};
use research::pipeline::{Pipeline, PipelineConfig};
use research::publish::{PublishConfig, Publisher};
use research::storage::ResearchIndex;
use research::twitter::BookmarkParser;
use tasks::ai::ProviderRegistry;

/// Research CLI - Monitor Twitter/X bookmarks and curate technical content.
#[derive(Parser)]
#[command(name = "research")]
#[command(about = "Twitter/X bookmark research pipeline")]
#[command(version)]
pub struct Cli {
    /// Enable verbose logging
    #[arg(short, long, global = true)]
    verbose: bool,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Run a single poll cycle (for CronJob use)
    Poll {
        /// Output directory for research
        #[arg(long, default_value = "/data/research")]
        output: PathBuf,

        /// State file path (tracks processed bookmarks)
        #[arg(long, default_value = "/data/state.json")]
        state: PathBuf,

        /// Minimum relevance score (0.0-1.0)
        #[arg(long, default_value = "0.5")]
        min_relevance: f32,

        /// Max bookmarks to process per run
        #[arg(long, default_value = "10")]
        batch_size: usize,

        /// AI model to use
        #[arg(long, default_value = "claude-sonnet-4-20250514")]
        model: String,

        /// Create a PR with new research entries
        #[arg(long)]
        create_pr: bool,

        /// GitHub repository for PR (owner/repo format)
        #[arg(long, default_value = "5dlabs/cto")]
        repo: String,

        /// Base branch for PR
        #[arg(long, default_value = "develop")]
        base_branch: String,

        /// Directory in repo for research files
        #[arg(long, default_value = "docs/research")]
        research_dir: String,
    },

    /// Interactive auth setup (run locally, not in container)
    Auth {
        /// Export cookies directly to Vault
        #[arg(long)]
        export_to_vault: bool,

        /// Output file for cookies (if not using Vault)
        #[arg(long)]
        output: Option<PathBuf>,
    },

    /// List recent research entries
    List {
        /// Filter by category
        #[arg(long)]
        category: Option<String>,

        /// Research directory
        #[arg(long, default_value = "./research")]
        dir: PathBuf,

        /// Limit results
        #[arg(long, default_value = "20")]
        limit: usize,
    },

    /// Search research entries
    Search {
        /// Search query
        query: String,

        /// Research directory
        #[arg(long, default_value = "./research")]
        dir: PathBuf,
    },

    /// Manually process a specific tweet URL
    Process {
        /// Tweet URL to process
        url: String,

        /// Output directory
        #[arg(long, default_value = "./research")]
        output: PathBuf,

        /// Force processing even if already seen
        #[arg(long)]
        force: bool,
    },

    /// Send email digest of recent research entries
    Digest {
        /// Research directory (contains index.json)
        #[arg(long, default_value = "./research")]
        dir: PathBuf,

        /// Digest state file path
        #[arg(long, default_value = "./digest-state.json")]
        state: PathBuf,

        /// Force send even if below threshold or recently sent
        #[arg(long)]
        force: bool,

        /// Check conditions and send only if threshold met (for cron)
        #[arg(long)]
        check_and_send: bool,

        /// Send a test email to verify configuration
        #[arg(long)]
        test: bool,

        /// Hours between scheduled digests (default: 24)
        #[arg(long, default_value = "24")]
        hours_between: u64,

        /// AI model to use for analysis
        #[arg(long, default_value = "claude-sonnet-4-20250514")]
        model: String,

        /// Skip AI analysis (just list entries)
        #[arg(long)]
        skip_analysis: bool,
    },
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    // Initialize tracing
    let filter = if cli.verbose {
        EnvFilter::new("research=debug,info")
    } else {
        EnvFilter::new("research=info,warn")
    };

    tracing_subscriber::registry()
        .with(fmt::layer())
        .with(filter)
        .init();

    match cli.command {
        Commands::Poll {
            output,
            state,
            min_relevance,
            batch_size,
            model,
            create_pr,
            repo,
            base_branch,
            research_dir,
        } => {
            tracing::info!(
                output = %output.display(),
                state = %state.display(),
                min_relevance,
                batch_size,
                model,
                create_pr,
                repo,
                "Starting poll cycle"
            );
            run_poll(
                output,
                state,
                min_relevance,
                batch_size,
                model,
                create_pr,
                repo,
                base_branch,
                research_dir,
            )
            .await
        }
        Commands::Auth {
            export_to_vault,
            output,
        } => {
            tracing::info!(export_to_vault, "Starting authentication");
            run_auth(export_to_vault, output).await
        }
        Commands::List {
            category,
            dir,
            limit,
        } => run_list(category, dir, limit).await,
        Commands::Search { query, dir } => run_search(&query, dir).await,
        Commands::Process { url, output, force } => {
            tracing::info!(url, force, "Processing tweet");
            run_process(&url, output, force).await
        }
        Commands::Digest {
            dir,
            state,
            force,
            check_and_send,
            test,
            hours_between,
            model,
            skip_analysis,
        } => {
            tracing::info!(
                dir = %dir.display(),
                state = %state.display(),
                force,
                check_and_send,
                test,
                model,
                skip_analysis,
                "Running digest command"
            );
            run_digest(dir, state, force, check_and_send, test, hours_between, model, skip_analysis).await
        }
    }
}

async fn run_poll(
    output: PathBuf,
    state: PathBuf,
    min_relevance: f32,
    batch_size: usize,
    model: String,
    create_pr: bool,
    repo: String,
    base_branch: String,
    research_dir: String,
) -> Result<()> {
    // Load session from environment
    let session = Session::from_env()?;
    tracing::debug!("Loaded session from environment");

    // Create AI provider
    let registry = ProviderRegistry::with_defaults();
    let provider = registry
        .get_for_model(&model)
        .ok_or_else(|| anyhow::anyhow!("No provider configured for model: {model}"))?;

    // Configure pipeline
    let config = PipelineConfig {
        output_dir: output.clone(),
        state_path: state.clone(),
        index_path: output.join("index.json"),
        min_relevance,
        batch_size,
        model,
        digest_state_path: Some(output.join("digest-state.json")),
    };

    // Run pipeline
    let pipeline = Pipeline::new(config, session, provider);
    let result = pipeline.poll_cycle().await?;

    // Print summary
    println!("\n📊 Poll Cycle Summary");
    println!("   Fetched: {}", result.fetched);
    println!("   Analyzed: {}", result.analyzed);
    println!("   Saved: {}", result.saved);
    println!("   Skipped: {}", result.skipped);

    if !result.errors.is_empty() {
        println!("   Errors: {}", result.errors.len());
        for err in &result.errors {
            tracing::warn!("Error: {err}");
        }
    }

    // Create PR if requested and there are saved entries
    if create_pr && result.saved > 0 {
        println!("\n📤 Creating pull request...");

        match create_research_pr(
            &output,
            &result.saved_ids,
            &repo,
            &base_branch,
            &research_dir,
        )
        .await
        {
            Ok(pr_url) => {
                println!("✅ Pull request created: {pr_url}");
            }
            Err(e) => {
                tracing::error!("Failed to create PR: {e}");
                // Don't fail the whole run if PR creation fails
            }
        }
    } else if create_pr && result.saved == 0 {
        println!("\n📭 No new entries to publish");
    }

    Ok(())
}

async fn create_research_pr(
    source_dir: &Path,
    entry_ids: &[String],
    repo: &str,
    base_branch: &str,
    research_dir: &str,
) -> Result<String> {
    let github_token = std::env::var("GITHUB_TOKEN")
        .or_else(|_| std::env::var("GH_TOKEN"))
        .map_err(|_| anyhow::anyhow!("GITHUB_TOKEN or GH_TOKEN not set for PR creation"))?;

    let config = PublishConfig {
        repo: repo.to_string(),
        base_branch: base_branch.to_string(),
        research_dir: research_dir.to_string(),
        github_token,
    };

    let publisher = Publisher::new(config)?;
    publisher.publish(source_dir, entry_ids).await
}

async fn run_auth(export_to_vault: bool, output: Option<PathBuf>) -> Result<()> {
    println!("🔐 Research - Twitter Authentication Setup\n");

    let auth = BrowserAuth::new(false);
    let session = auth.login().await?;

    if export_to_vault {
        println!("\n📤 Exporting to Vault...");
        println!("   TWITTER_AUTH_TOKEN={}", session.auth_token);
        if let Some(ct0) = &session.ct0 {
            println!("   TWITTER_CT0={ct0}");
        }
        println!("\n   To add to Vault, run:");
        println!(
            "   vault kv put secret/research-twitter auth_token={}",
            session.auth_token
        );
    } else {
        let output_path = output.unwrap_or_else(|| PathBuf::from(".twitter-session.json"));
        session.save(&output_path)?;
        println!("✅ Session saved to: {}", output_path.display());
    }

    Ok(())
}

async fn run_list(category: Option<String>, dir: PathBuf, limit: usize) -> Result<()> {
    let index_path = dir.join("index.json");
    let index = ResearchIndex::load(&index_path)?;

    println!("📋 Research entries in {}\n", dir.display());

    let entries: Vec<_> = if let Some(cat_str) = &category {
        if let Some(cat) = Category::parse(cat_str) {
            index.by_category(cat)
        } else {
            println!("Unknown category: {cat_str}");
            println!("Available: agents, rust, infrastructure, tooling, architecture, devops, security, research, announcements, other");
            return Ok(());
        }
    } else {
        index.recent(limit)
    };

    if entries.is_empty() {
        println!("No entries found.");
        return Ok(());
    }

    for entry in entries.iter().take(limit) {
        let categories: Vec<_> = entry.categories.iter().map(ToString::to_string).collect();
        println!(
            "🔖 {} @{} ({})",
            entry.id,
            entry.author,
            categories.join(", ")
        );
        println!("   {}", entry.preview);
        println!(
            "   Score: {:.2} | {}\n",
            entry.score,
            entry.processed_at.format("%Y-%m-%d")
        );
    }

    println!("Total: {} entries", entries.len().min(limit));

    Ok(())
}

async fn run_search(query: &str, dir: PathBuf) -> Result<()> {
    let index_path = dir.join("index.json");
    let index = ResearchIndex::load(&index_path)?;

    println!("🔍 Searching for: {query}\n");

    let results = index.search(query);

    if results.is_empty() {
        println!("No results found.");
        return Ok(());
    }

    for entry in &results {
        let categories: Vec<_> = entry.categories.iter().map(ToString::to_string).collect();
        println!(
            "🔖 {} @{} ({})",
            entry.id,
            entry.author,
            categories.join(", ")
        );
        println!("   {}", entry.preview);
        println!(
            "   Score: {:.2} | {}\n",
            entry.score,
            entry.processed_at.format("%Y-%m-%d")
        );
    }

    println!("Found: {} results", results.len());

    Ok(())
}

async fn run_process(url: &str, output: PathBuf, force: bool) -> Result<()> {
    println!("🔖 Processing tweet: {url}\n");
    println!("Output: {}", output.display());
    println!("Force: {force}");

    // Extract tweet ID
    let tweet_id = BookmarkParser::extract_tweet_id(url)
        .ok_or_else(|| anyhow::anyhow!("Invalid tweet URL: {url}"))?;

    println!("Tweet ID: {tweet_id}");

    // TODO: Implement single tweet processing
    // 1. Fetch tweet content using browser automation
    // 2. Analyze relevance
    // 3. Enrich links
    // 4. Store as markdown

    println!("\n⚠️  Single tweet processing not yet implemented.");
    println!("   Use the 'poll' command to process bookmarks.");

    Ok(())
}

#[allow(clippy::fn_params_excessive_bools)] // CLI flags are naturally bools
async fn run_digest(
    dir: PathBuf,
    state_path: PathBuf,
    force: bool,
    check_and_send: bool,
    test: bool,
    hours_between: u64,
    model: String,
    skip_analysis: bool,
) -> Result<()> {
    println!("📬 Research Digest\n");

    // If test mode, just send a test email
    if test {
        println!("Sending test email...");
        let sender = EmailSender::from_env()?;
        sender.send_test().await?;
        println!("✅ Test email sent successfully!");
        return Ok(());
    }

    // Load digest config
    let config = DigestConfig::from_env()?;
    println!("   To: {}", config.to_email);
    println!("   Burst threshold: {}", config.burst_threshold);
    println!("   Min for digest: {}", config.min_for_digest);

    // Load digest state
    let mut digest_state = DigestState::load(&state_path)?;
    println!(
        "   Pending entries: {}",
        digest_state.pending_count()
    );
    if let Some(last) = digest_state.last_digest_at {
        println!("   Last digest: {}", last.format("%Y-%m-%d %H:%M UTC"));
    } else {
        println!("   Last digest: never");
    }

    // Load research index
    let index_path = dir.join("index.json");
    let index = ResearchIndex::load(&index_path)?;
    println!("   Total entries in index: {}", index.entries.len());

    // Determine which entries to include
    // If no pending entries tracked, use recent entries from index
    let entry_ids: Vec<String> = if digest_state.entries_since_digest.is_empty() {
        // First run or state was cleared - grab recent entries
        index
            .recent(20)
            .into_iter()
            .map(|e| e.id.clone())
            .collect()
    } else {
        digest_state.entries_since_digest.clone()
    };

    let pending_count = entry_ids.len();
    println!("\n   Entries to digest: {pending_count}");

    // Check if we should send
    let should_send = if force {
        println!("   Force mode: sending regardless of thresholds");
        true
    } else if check_and_send {
        // Cron mode: check both threshold and time
        let burst_triggered = config.should_burst_send(pending_count);
        let scheduled_time = digest_state.is_scheduled_time(hours_between);
        let has_enough = config.has_enough_for_digest(pending_count);

        if burst_triggered {
            println!("   ✓ Burst threshold reached ({pending_count} >= {})", config.burst_threshold);
            true
        } else if scheduled_time && has_enough {
            println!("   ✓ Scheduled time reached and have enough entries");
            true
        } else {
            if !scheduled_time {
                println!("   ✗ Not yet time for scheduled digest (< {hours_between}h since last)");
            }
            if !has_enough {
                println!(
                    "   ✗ Not enough entries ({pending_count} < {})",
                    config.min_for_digest
                );
            }
            false
        }
    } else {
        // Default: require minimum entries
        if config.has_enough_for_digest(pending_count) {
            true
        } else {
            println!(
                "   Not enough entries ({pending_count} < {}). Use --force to send anyway.",
                config.min_for_digest
            );
            false
        }
    };

    if !should_send {
        println!("\n📭 No digest sent");
        return Ok(());
    }

    // Gather entries for digest
    let entries: Vec<_> = entry_ids
        .iter()
        .filter_map(|id| index.entries.get(id))
        .collect();

    if entries.is_empty() {
        println!("\n⚠️  No entries found in index for pending IDs");
        return Ok(());
    }

    // Run AI analysis if not skipped
    let analysis = if skip_analysis {
        println!("\n⏭️  Skipping AI analysis");
        None
    } else {
        println!("\n🤖 Running AI analysis with {model}...");
        
        // Create AI provider
        let registry = ProviderRegistry::with_defaults();
        let provider = registry
            .get_for_model(&model)
            .ok_or_else(|| anyhow::anyhow!("No provider configured for model: {model}"))?;
        
        let analyzer = DigestAnalyzer::new(provider, model.clone());
        match analyzer.analyze(&entries).await {
            Ok(a) => {
                println!("   ✓ Analysis complete");
                println!("   Summary: {}", &a.summary[..a.summary.len().min(100)]);
                println!("   High priority items: {}", a.high_priority.len());
                println!("   Worth investigating: {}", a.worth_investigating.len());
                println!("   Tips & tricks: {}", a.tips_and_tricks.len());
                Some(a)
            }
            Err(e) => {
                tracing::warn!("AI analysis failed: {e}");
                println!("   ⚠️  Analysis failed, sending without recommendations");
                None
            }
        }
    };

    // Generate email content
    let now = chrono::Utc::now();
    let html = DigestGenerator::generate_html(&entries, analysis.as_ref(), now);
    let text = DigestGenerator::generate_text(&entries, analysis.as_ref(), now);
    let subject = format!(
        "CTO Research Digest - {} new entries",
        entries.len()
    );

    // Send email
    println!("\n📤 Sending digest email...");
    let sender = EmailSender::new(config);
    sender.send(&subject, &html, &text).await?;

    // Update state
    digest_state.mark_digest_sent();
    digest_state.save(&state_path)?;

    println!("✅ Digest sent successfully!");
    println!("   Entries included: {}", entries.len());
    println!("   Total digests sent: {}", digest_state.total_digests_sent);

    Ok(())
}
