use anyhow::{Context, Result};
use std::path::Path;
use tokio::process::Command;

/// Find play.lobster.yaml in the repo's .tasks directory
pub fn find_play_yaml(repo_path: &Path) -> Result<std::path::PathBuf> {
    let candidates = [
        repo_path.join(".tasks/docs/play.lobster.yaml"),
        repo_path.join(".tasks/play.lobster.yaml"),
        repo_path.join("play.lobster.yaml"),
    ];
    for c in &candidates {
        if c.exists() {
            return Ok(c.clone());
        }
    }
    anyhow::bail!(
        "play.lobster.yaml not found in {}. Searched:\n  {}",
        repo_path.display(),
        candidates
            .iter()
            .map(|c| c.display().to_string())
            .collect::<Vec<_>>()
            .join("\n  ")
    );
}

/// Find play-config.yaml in the repo's .tasks directory
pub fn find_play_config(repo_path: &Path) -> Result<std::path::PathBuf> {
    let candidates = [
        repo_path.join(".tasks/play-config.yaml"),
        repo_path.join(".tasks/docs/play-config.yaml"),
    ];
    for c in &candidates {
        if c.exists() {
            return Ok(c.clone());
        }
    }
    anyhow::bail!(
        "play-config.yaml not found in {}. Searched:\n  {}",
        repo_path.display(),
        candidates
            .iter()
            .map(|c| c.display().to_string())
            .collect::<Vec<_>>()
            .join("\n  ")
    );
}

/// Execute lobster run with the given play YAML and args JSON
pub async fn run_lobster(
    play_yaml: &Path,
    args_json: &serde_json::Value,
    kubeconfig: Option<&str>,
    dry_run: bool,
) -> Result<()> {
    let args_str = serde_json::to_string(args_json)?;

    if dry_run {
        println!("=== DRY RUN ===");
        println!("Play:      {}", play_yaml.display());
        println!("Args JSON: {}", serde_json::to_string_pretty(args_json)?);
        if let Some(kc) = kubeconfig {
            println!("Kubeconfig: {}", kc);
        }
        println!("===============");
        println!("\nWould execute:");
        println!(
            "  lobster run {} --args-json '{}'",
            play_yaml.display(),
            args_str
        );
        return Ok(());
    }

    println!("Launching play: {}", play_yaml.display());
    println!(
        "  Provider: {} | Model: {} | CLI: {}",
        args_json["provider"], args_json["model"], args_json["cli"]
    );
    println!(
        "  Namespace: {} | Repo: {}",
        args_json["namespace"], args_json["repo_url"]
    );
    if args_json["discord_enabled"] == "true" {
        println!("  Discord notifications: enabled");
    }
    println!();

    let mut cmd = Command::new("lobster");
    cmd.arg("run")
        .arg(play_yaml.as_os_str())
        .arg("--args-json")
        .arg(&args_str);

    if let Some(kc) = kubeconfig {
        cmd.env("KUBECONFIG", kc);
    }

    let status = cmd
        .status()
        .await
        .context("Failed to execute lobster. Is it installed? (npm i -g @clawdbot/lobster)")?;

    if !status.success() {
        anyhow::bail!("lobster exited with status {}", status);
    }

    Ok(())
}
