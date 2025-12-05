//! Research CLI - Twitter/X bookmark research pipeline.

use anyhow::Result;
use clap::{Parser, Subcommand};
use std::path::{Path, PathBuf};
use tracing_subscriber::{fmt, prelude::*, EnvFilter};

use research::analysis::Category;
use research::auth::{BrowserAuth, Session};
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
        #[arg(long, default_value = "main")]
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
            eprintln!("     - {err}");
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
                eprintln!("❌ Failed to create PR: {e}");
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
