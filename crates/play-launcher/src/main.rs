mod config;
mod launcher;

use anyhow::{Context, Result};
use clap::Parser;
use std::collections::HashMap;
use std::path::PathBuf;

use config::{build_args_json, CtoConfig, PlayConfig};
use launcher::{find_play_config, find_play_yaml, run_lobster};

#[derive(Parser, Debug)]
#[command(
    name = "cto-play",
    about = "Launch CTO play.lobster.yaml workflows with merged defaults",
    long_about = "Reads per-repo .tasks/play-config.yaml and CTO platform defaults,\n\
                  merges them with any CLI overrides, and invokes lobster run."
)]
struct Cli {
    /// Path to the repo root (default: current directory)
    #[arg(long, default_value = ".")]
    repo_path: PathBuf,

    /// Path to CTO config JSON (default: /etc/cto/config.json or CTO_CONFIG env)
    #[arg(long)]
    cto_config: Option<PathBuf>,

    /// Override kubeconfig path
    #[arg(long)]
    kubeconfig: Option<String>,

    /// Override Kubernetes namespace
    #[arg(long)]
    namespace: Option<String>,

    /// Override inference provider
    #[arg(long)]
    provider: Option<String>,

    /// Override model
    #[arg(long)]
    model: Option<String>,

    /// Override coding CLI
    #[arg(long)]
    cli: Option<String>,

    /// Override harness agent (openclaw/hermes)
    #[arg(long)]
    harness_agent: Option<String>,

    /// Override repo URL
    #[arg(long)]
    repo_url: Option<String>,

    /// Enable Discord notifications
    #[arg(long)]
    discord: Option<bool>,

    /// Discord bridge URL
    #[arg(long)]
    discord_bridge_url: Option<String>,

    /// Linear session ID
    #[arg(long)]
    linear_session_id: Option<String>,

    /// Linear team ID
    #[arg(long)]
    linear_team_id: Option<String>,

    /// Override working directory
    #[arg(long)]
    working_directory: Option<String>,

    /// Override auto-merge
    #[arg(long)]
    auto_merge: Option<bool>,

    /// Show what would be passed to lobster without executing
    #[arg(long)]
    dry_run: bool,
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    let repo_path = cli
        .repo_path
        .canonicalize()
        .context("Could not resolve repo path")?;

    // Load per-repo play config
    let play_config_path = find_play_config(&repo_path)?;
    let play_config = PlayConfig::load(&play_config_path)
        .with_context(|| format!("Failed to parse {}", play_config_path.display()))?;
    eprintln!("Loaded play config: {}", play_config_path.display());

    // Load CTO config (optional — falls back to play-config defaults only)
    let cto_config = load_cto_config(&cli.cto_config);
    if cto_config.is_some() {
        eprintln!("Loaded CTO config");
    }

    // Build overrides from CLI flags
    let mut overrides = HashMap::new();
    if let Some(v) = &cli.namespace {
        overrides.insert("namespace".into(), v.clone());
    }
    if let Some(v) = &cli.provider {
        overrides.insert("provider".into(), v.clone());
    }
    if let Some(v) = &cli.model {
        overrides.insert("model".into(), v.clone());
    }
    if let Some(v) = &cli.cli {
        overrides.insert("cli".into(), v.clone());
    }
    if let Some(v) = &cli.harness_agent {
        overrides.insert("harness_agent".into(), v.clone());
    }
    if let Some(v) = &cli.repo_url {
        overrides.insert("repo_url".into(), v.clone());
    }
    if let Some(v) = cli.discord {
        overrides.insert("discord_enabled".into(), v.to_string());
    }
    if let Some(v) = &cli.discord_bridge_url {
        overrides.insert("discord_bridge_url".into(), v.clone());
    }
    if let Some(v) = &cli.linear_session_id {
        overrides.insert("linear_session_id".into(), v.clone());
    }
    if let Some(v) = &cli.linear_team_id {
        overrides.insert("linear_team_id".into(), v.clone());
    }
    if let Some(v) = &cli.working_directory {
        overrides.insert("working_directory".into(), v.clone());
    }
    if let Some(v) = cli.auto_merge {
        overrides.insert("auto_merge".into(), v.to_string());
    }

    // Merge configs and build args JSON
    let args_json = build_args_json(&play_config, &cto_config, &overrides);

    // Find and run play.lobster.yaml
    let play_yaml = find_play_yaml(&repo_path)?;

    let kubeconfig_path = cli
        .kubeconfig
        .as_deref()
        .or_else(|| {
            let kc = &play_config.kubeconfig.path;
            if kc.is_empty() {
                None
            } else {
                Some(kc.as_str())
            }
        });

    run_lobster(&play_yaml, &args_json, kubeconfig_path, cli.dry_run).await?;

    Ok(())
}

fn load_cto_config(explicit_path: &Option<PathBuf>) -> Option<CtoConfig> {
    // Priority: explicit flag > CTO_CONFIG env > /etc/cto/config.json > ./cto-config.json
    let candidates: Vec<PathBuf> = if let Some(p) = explicit_path {
        vec![p.clone()]
    } else {
        let mut c = Vec::new();
        if let Ok(env_path) = std::env::var("CTO_CONFIG") {
            c.push(PathBuf::from(env_path));
        }
        c.push(PathBuf::from("/etc/cto/config.json"));
        c.push(PathBuf::from("cto-config.json"));
        c
    };

    for path in candidates {
        if path.exists() {
            match CtoConfig::load(&path) {
                Ok(config) => return Some(config),
                Err(e) => {
                    eprintln!("Warning: could not parse CTO config at {}: {}", path.display(), e);
                }
            }
        }
    }
    None
}
