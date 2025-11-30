//! CTO MCP Server
//!
//! This module provides the Model Context Protocol (MCP) server for the CTO platform.

#![allow(clippy::map_unwrap_or)]
#![allow(clippy::format_push_string)]
#![allow(clippy::no_effect_underscore_binding)]

use anyhow::{anyhow, Context, Result};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::collections::{BTreeMap, HashMap};
use std::io::Write;
use std::path::Path;
use std::process::Command;
use std::sync::OnceLock;
use tempfile::NamedTempFile;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::runtime::Runtime;
use tokio::signal;
use tokio::time::{timeout, Duration};

mod doc_proxy;
mod tools;

#[cfg(test)]
mod model_validation_tests;

// Global configuration loaded once at startup
static CTO_CONFIG: OnceLock<CtoConfig> = OnceLock::new();

#[derive(Debug, Deserialize, Clone)]
struct AgentConfig {
    #[serde(rename = "githubApp")]
    github_app: String,
    cli: String,
    model: String,
    #[serde(default)]
    #[allow(dead_code)]
    tools: Option<AgentTools>,
    #[serde(default, rename = "maxRetries")]
    max_retries: Option<u32>,
    #[serde(default, rename = "modelRotation")]
    model_rotation: Option<ModelRotationConfig>,
    #[serde(default)]
    #[allow(dead_code)]
    features: Option<AgentFeatures>,
}

#[derive(Debug, Deserialize, Clone, Default)]
struct AgentFeatures {
    /// Enable Effect.ts integration with effect-solutions CLI
    #[serde(default, rename = "effectSolutions")]
    #[allow(dead_code)]
    effect_solutions: bool,
}

#[derive(Debug, Deserialize, Clone)]
struct ModelRotationConfig {
    #[serde(default)]
    enabled: bool,
    #[serde(default)]
    models: Vec<String>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
struct AgentTools {
    #[serde(default)]
    #[allow(dead_code)]
    remote: Vec<String>,
    #[serde(default, rename = "localServers")]
    #[allow(dead_code)]
    local_servers: Option<BTreeMap<String, ServerConfig>>,
}

// No predefined server names in code. Servers are defined only via config.

#[derive(Debug, Deserialize, Serialize, Clone)]
struct ServerConfig {
    #[allow(dead_code)]
    enabled: bool,
    #[allow(dead_code)]
    tools: Vec<String>,
}

#[derive(Debug, Deserialize, Clone)]
struct CtoConfig {
    version: String,
    defaults: WorkflowDefaults,
    agents: HashMap<String, AgentConfig>,
}

#[derive(Debug, Deserialize, Clone)]
struct WorkflowDefaults {
    docs: DocsDefaults,
    #[allow(dead_code)]
    code: CodeDefaults,
    #[serde(default)]
    intake: IntakeDefaults,
    #[serde(default)]
    play: PlayDefaults,
}

#[derive(Debug, Deserialize, Clone)]
struct DocsDefaults {
    model: String,
    #[serde(rename = "githubApp")]
    #[allow(dead_code)] // Used for backwards compatibility, will be removed in future version
    github_app: String,
    #[serde(rename = "includeCodebase")]
    include_codebase: bool,
    #[serde(rename = "sourceBranch")]
    #[allow(dead_code)] // Used for backwards compatibility, will be removed in future version
    source_branch: String,
}

#[derive(Debug, Deserialize, Clone)]
struct CodeDefaults {
    #[allow(dead_code)]
    model: String,
    #[serde(rename = "githubApp")]
    #[allow(dead_code)]
    github_app: String,
    #[serde(rename = "continueSession")]
    #[allow(dead_code)]
    continue_session: bool,
    #[serde(rename = "workingDirectory")]
    #[allow(dead_code)]
    working_directory: String,
    #[serde(rename = "overwriteMemory")]
    #[allow(dead_code)]
    overwrite_memory: bool,
    #[allow(dead_code)]
    repository: Option<String>,
    #[serde(rename = "docsRepository")]
    #[allow(dead_code)]
    docs_repository: Option<String>,
    #[serde(rename = "docsProjectDirectory")]
    #[allow(dead_code)]
    docs_project_directory: Option<String>,
    #[allow(dead_code)]
    service: Option<String>,
    #[serde(rename = "maxRetries")]
    #[allow(dead_code)]
    max_retries: Option<u32>,
    #[serde(default)]
    #[allow(dead_code)]
    cli: Option<String>,
}

#[derive(Debug, Deserialize, Clone)]
struct ModelConfig {
    model: String,
    provider: String,
}

#[derive(Debug, Deserialize, Clone)]
struct IntakeDefaults {
    #[serde(rename = "githubApp")]
    github_app: String,
    primary: ModelConfig,
    research: ModelConfig,
    fallback: ModelConfig,
}

impl Default for IntakeDefaults {
    fn default() -> Self {
        // No defaults - require explicit configuration
        IntakeDefaults {
            github_app: String::new(),
            primary: ModelConfig {
                model: String::new(),
                provider: String::new(),
            },
            research: ModelConfig {
                model: String::new(),
                provider: String::new(),
            },
            fallback: ModelConfig {
                model: String::new(),
                provider: String::new(),
            },
        }
    }
}

/// Validate model name format (permissive - allows any reasonable model name)
fn validate_model_name(model: &str) -> Result<()> {
    // Simple validation: reject empty or obviously invalid names
    if model.trim().is_empty() {
        return Err(anyhow!("Model name cannot be empty"));
    }

    // Allow any non-empty model name - let the CLI handle model-specific validation
    Ok(())
}

#[derive(Debug, Deserialize, Clone, Default)]
struct PlayDefaults {
    model: String,
    cli: String,
    #[serde(rename = "implementationAgent")]
    implementation_agent: String,
    #[serde(rename = "frontendAgent")]
    frontend_agent: Option<String>,
    #[serde(rename = "qualityAgent")]
    quality_agent: String,
    #[serde(rename = "securityAgent")]
    security_agent: String,
    #[serde(rename = "testingAgent")]
    testing_agent: String,
    repository: Option<String>,
    service: Option<String>,
    #[serde(rename = "docsRepository")]
    docs_repository: Option<String>,
    #[serde(rename = "docsProjectDirectory")]
    docs_project_directory: Option<String>,
    #[serde(rename = "workingDirectory")]
    #[allow(dead_code)]
    // Still in config for backward compatibility, but we use docs_project_directory for tasks
    working_directory: Option<String>,
    #[serde(rename = "maxRetries")]
    max_retries: Option<u32>,
    #[serde(rename = "implementationMaxRetries")]
    implementation_max_retries: Option<u32>,
    #[serde(rename = "qualityMaxRetries")]
    quality_max_retries: Option<u32>,
    #[serde(rename = "securityMaxRetries")]
    security_max_retries: Option<u32>,
    #[serde(rename = "testingMaxRetries")]
    testing_max_retries: Option<u32>,
    #[serde(rename = "frontendMaxRetries")]
    frontend_max_retries: Option<u32>,
    #[serde(rename = "autoMerge")]
    auto_merge: Option<bool>,
    #[serde(rename = "parallelExecution")]
    parallel_execution: Option<bool>,
}

/// Load configuration from cto-config.json file
/// Looks in current directory, workspace root, or `WORKSPACE_FOLDER_PATHS` for cto-config.json
#[allow(clippy::disallowed_macros)]
fn load_cto_config() -> Result<CtoConfig> {
    let mut config_paths = vec![
        std::path::PathBuf::from("cto-config.json"),
        std::path::PathBuf::from("../cto-config.json"),
    ];

    // Add workspace folder paths if available (Cursor provides this)
    if let Ok(workspace_paths) = std::env::var("WORKSPACE_FOLDER_PATHS") {
        for workspace_path in workspace_paths.split(',') {
            let workspace_path = workspace_path.trim();
            config_paths.push(std::path::PathBuf::from(workspace_path).join("cto-config.json"));
        }
    }

    for config_path in config_paths {
        if config_path.exists() {
            eprintln!("📋 Loading configuration from: {}", config_path.display());
            let config_content = std::fs::read_to_string(&config_path).with_context(|| {
                format!("Failed to read config file: {}", config_path.display())
            })?;

            let config: CtoConfig = serde_json::from_str(&config_content).with_context(|| {
                format!("Failed to parse config file: {}", config_path.display())
            })?;

            // Basic version validation
            if config.version != "1.0" {
                return Err(anyhow!(
                    "Unsupported config version: {}. Expected: 1.0",
                    config.version
                ));
            }

            eprintln!("✅ Configuration loaded successfully");
            return Ok(config);
        }
    }

    let workspace_info = if let Ok(workspace_paths) = std::env::var("WORKSPACE_FOLDER_PATHS") {
        format!(" Also checked workspace folders: {workspace_paths}")
    } else {
        " No WORKSPACE_FOLDER_PATHS environment variable found (Cursor-only feature).".to_string()
    };

    Err(anyhow!("cto-config.json not found in current directory or parent directory.{workspace_info} Please create a configuration file in your project root."))
}

/// Load repository-specific configuration from cto-config.json
/// Used during workflow creation to get repository's agent tool configurations
#[allow(clippy::disallowed_macros)]
fn load_repository_config(repository_path: Option<&str>) -> Option<CtoConfig> {
    // Try to load from explicit repository path first
    if let Some(repo_path) = repository_path {
        let config_path = std::path::PathBuf::from(repo_path).join("cto-config.json");

        if config_path.exists() {
            eprintln!(
                "📋 Loading repository config from: {}",
                config_path.display()
            );

            match std::fs::read_to_string(&config_path) {
                Ok(config_content) => match serde_json::from_str::<CtoConfig>(&config_content) {
                    Ok(config) => {
                        if config.version == "1.0" {
                            eprintln!("✅ Repository configuration loaded successfully");
                            eprintln!("   Agents defined: {}", config.agents.len());
                            return Some(config);
                        }
                        eprintln!(
                            "⚠️  Repository config version mismatch: {} (expected 1.0)",
                            config.version
                        );
                    }
                    Err(e) => {
                        eprintln!("⚠️  Failed to parse repository config: {e}");
                    }
                },
                Err(e) => {
                    eprintln!("⚠️  Failed to read repository config: {e}");
                }
            }
        } else {
            eprintln!(
                "ℹ️  No cto-config.json found in repository at: {}",
                config_path.display()
            );
        }
    }

    // Try workspace detection as fallback
    if let Some(workspace_path) = resolve_workspace_dir() {
        let config_path = workspace_path.join("cto-config.json");

        if config_path.exists() {
            eprintln!(
                "📋 Loading repository config from workspace: {}",
                config_path.display()
            );

            match std::fs::read_to_string(&config_path) {
                Ok(config_content) => match serde_json::from_str::<CtoConfig>(&config_content) {
                    Ok(config) => {
                        if config.version == "1.0" {
                            eprintln!("✅ Repository configuration loaded from workspace");
                            eprintln!("   Agents defined: {}", config.agents.len());
                            return Some(config);
                        }
                    }
                    Err(e) => {
                        eprintln!("⚠️  Failed to parse workspace config: {e}");
                    }
                },
                Err(e) => {
                    eprintln!("⚠️  Failed to read workspace config: {e}");
                }
            }
        }
    }

    eprintln!("ℹ️  Using platform default configuration (no repository config found)");
    None
}

#[derive(Deserialize)]
struct RpcRequest {
    id: Option<Value>,
    method: String,
    params: Option<Value>,
}

#[derive(Serialize)]
struct RpcSuccessResponse {
    jsonrpc: String,
    result: Value,
    id: Option<Value>,
}

#[derive(Debug, Serialize)]
struct RpcError {
    code: i32,
    message: String,
    data: Option<Value>,
}

#[derive(Serialize)]
struct RpcErrorResponse {
    jsonrpc: String,
    error: RpcError,
    id: Option<Value>,
}

fn extract_params(params: Option<&Value>) -> HashMap<String, Value> {
    params
        .and_then(|p| p.as_object())
        .map(|obj| obj.iter().map(|(k, v)| (k.clone(), v.clone())).collect())
        .unwrap_or_default()
}

#[allow(clippy::cast_possible_truncation)]
fn parse_max_retries_argument(arguments: &HashMap<String, Value>, key: &str) -> Option<u32> {
    arguments.get(key).and_then(|value| match value {
        Value::Number(num) => num.as_u64().map(|v| v as u32),
        Value::String(s) => s.parse::<u32>().ok(),
        _ => None,
    })
}

fn parse_bool_argument(arguments: &HashMap<String, Value>, key: &str) -> Option<bool> {
    arguments.get(key).and_then(|value| match value {
        Value::Bool(b) => Some(*b),
        Value::String(s) => match s.to_lowercase().as_str() {
            "true" | "yes" | "1" => Some(true),
            "false" | "no" | "0" => Some(false),
            _ => None,
        },
        _ => None,
    })
}

fn resolve_workspace_dir() -> Option<std::path::PathBuf> {
    // 1. Try current directory first - this is most likely the intended workspace
    if let Ok(cwd) = std::env::current_dir() {
        // Verify it's a valid workspace (has .git or cto-config.json)
        if cwd.join(".git").exists() || cwd.join("cto-config.json").exists() {
            return Some(cwd);
        }
    }

    // 2. Check WORKSPACE_FOLDER_PATHS as fallback (Cursor environment)
    if let Ok(paths_str) = std::env::var("WORKSPACE_FOLDER_PATHS") {
        let paths: Vec<&str> = paths_str
            .split(',')
            .map(str::trim)
            .filter(|s| !s.is_empty())
            .collect();

        // If only one path, use it
        if paths.len() == 1 {
            return Some(std::path::PathBuf::from(paths[0]));
        }

        // Multiple paths - try to find one with cto-config.json
        for path_str in &paths {
            let path = std::path::PathBuf::from(path_str);
            if path.join("cto-config.json").exists() {
                return Some(path);
            }
        }

        // No path has cto-config.json - return None to signal ambiguity
        // Callers should handle this by requiring explicit configuration
    }

    // 3. No valid workspace found
    None
}

fn handle_mcp_methods(method: &str, _params_map: &HashMap<String, Value>) -> Option<Result<Value>> {
    match method {
        "initialize" => Some(Ok(json!({
            "protocolVersion": "2025-06-18",
            "capabilities": {
                "tools": {
                    "listChanged": true
                }
            },
            "serverInfo": {
                "name": "cto-mcp",
                "title": "Agent Platform MCP Server",
                "version": "1.0.0",
                "buildTimestamp": env!("BUILD_TIMESTAMP")
            }
        }))),
        "tools/list" => {
            // Get config if available to show dynamic agent options
            match CTO_CONFIG.get() {
                Some(config) => Some(Ok(tools::get_tool_schemas_with_config(&config.agents))),
                None => Some(Ok(tools::get_tool_schemas())),
            }
        }
        _ => None,
    }
}

fn find_command(name: &str) -> String {
    // Check common installation locations in order
    let common_paths = [
        format!("/opt/homebrew/bin/{name}"), // Homebrew Apple Silicon
        format!("/usr/local/bin/{name}"),    // Homebrew Intel / standard Linux
        format!("/usr/bin/{name}"),          // System binaries
        name.to_string(),                    // Fallback to PATH
    ];

    for path in &common_paths {
        if std::path::Path::new(path).exists() {
            return path.clone();
        }
    }

    // If nothing found, return the name and let PATH resolution happen
    name.to_string()
}

fn run_argo_cli(args: &[&str]) -> Result<String> {
    let argo_cmd = find_command("argo");
    let output = Command::new(&argo_cmd)
        .args(args)
        .output()
        .context("Failed to execute argo command")?;

    if output.status.success() {
        Ok(String::from_utf8(output.stdout)?.trim().to_string())
    } else {
        let stderr = String::from_utf8(output.stderr)?;
        Err(anyhow!("Argo command failed: {stderr}"))
    }
}

/// Get the remote URL for the current git repository
#[allow(dead_code)] // Deprecated: use get_git_repository_url_in_dir instead
fn get_git_remote_url() -> Result<String> {
    let output = Command::new("git")
        .args(["remote", "get-url", "origin"])
        .output()
        .context("Failed to execute git command")?;

    if output.status.success() {
        let url = String::from_utf8(output.stdout)?.trim().to_string();

        // Convert SSH URLs to HTTPS format
        if url.starts_with("git@github.com:") {
            let repo_path = url.strip_prefix("git@github.com:").unwrap();
            let repo_path = repo_path.strip_suffix(".git").unwrap_or(repo_path);
            Ok(format!("https://github.com/{repo_path}"))
        } else {
            Ok(url)
        }
    } else {
        let stderr = String::from_utf8(output.stderr)?;
        Err(anyhow!("Git command failed: {stderr}"))
    }
}

/// Get the current git branch in a specific directory
fn get_git_current_branch_in_dir(dir: Option<&Path>) -> Result<String> {
    let mut cmd = Command::new("git");
    cmd.args(["branch", "--show-current"]);

    if let Some(dir) = dir {
        cmd.current_dir(dir);
    }

    let output = cmd.output().context("Failed to execute git command")?;

    if output.status.success() {
        let branch = String::from_utf8(output.stdout)?.trim().to_string();
        if branch.is_empty() {
            Ok("main".to_string()) // fallback to main if no branch (detached HEAD)
        } else {
            Ok(branch)
        }
    } else {
        let stderr = String::from_utf8(output.stderr)?;
        Err(anyhow!("Git command failed: {stderr}"))
    }
}

/// Get the current git repository URL in org/repo format from a specific directory
fn get_git_repository_url_in_dir(dir: Option<&Path>) -> Result<String> {
    let mut cmd = Command::new("git");
    cmd.args(["remote", "get-url", "origin"]);

    if let Some(dir) = dir {
        cmd.current_dir(dir);
    }

    let output = cmd
        .output()
        .context("Failed to execute git remote command")?;

    if !output.status.success() {
        let stderr = String::from_utf8(output.stderr)?;
        return Err(anyhow!("Failed to get git repository URL: {stderr}"));
    }

    let url = String::from_utf8(output.stdout)?.trim().to_string();

    // Parse GitHub URL to get org/repo format
    // Handles both https://github.com/org/repo.git and git@github.com:org/repo.git
    if url.contains("github.com/") {
        // https format: https://github.com/org/repo.git
        let parts: Vec<&str> = url.split("github.com/").collect();
        if parts.len() > 1 {
            let org_repo = parts[1].trim_end_matches(".git");
            return Ok(org_repo.to_string());
        }
    } else if url.contains("github.com:") {
        // SSH format: git@github.com:org/repo.git
        let parts: Vec<&str> = url.split("github.com:").collect();
        if parts.len() > 1 {
            let org_repo = parts[1].trim_end_matches(".git");
            return Ok(org_repo.to_string());
        }
    }

    Err(anyhow!("Could not parse repository URL: {url}"))
}

/// Validate repository URL format
fn validate_repository_url(repo_url: &str) -> Result<()> {
    // Accept both formats: "org/repo" and "https://github.com/org/repo"
    if repo_url.starts_with("https://github.com/") {
        // Validate HTTPS URL format
        let path = repo_url.trim_start_matches("https://github.com/");
        let parts: Vec<&str> = path.trim_end_matches(".git").split('/').collect();
        if parts.len() != 2 || parts[0].is_empty() || parts[1].is_empty() {
            return Err(anyhow!(
                "Repository URL must be in format 'https://github.com/org/repo'"
            ));
        }
    } else {
        // Validate org/repo format
        let parts: Vec<&str> = repo_url.split('/').collect();
        if parts.len() != 2 || parts[0].is_empty() || parts[1].is_empty() {
            return Err(anyhow!(
                "Repository must be in format 'org/repo' or 'https://github.com/org/repo'"
            ));
        }
    }

    Ok(())
}

/// DEPRECATED: Use `handle_intake_workflow` instead. This function is kept for backwards compatibility.
#[allow(dead_code)]
#[allow(clippy::disallowed_macros, clippy::too_many_lines)]
fn handle_docs_workflow(arguments: &HashMap<String, Value>) -> Result<Value> {
    let working_directory = arguments
        .get("working_directory")
        .and_then(|v| v.as_str())
        .ok_or(anyhow!("Missing required parameter: working_directory"))?;

    let config = CTO_CONFIG.get().unwrap();

    // Get workspace directory from Cursor environment, then navigate to working_directory
    let workspace_dir = resolve_workspace_dir().unwrap_or_else(|| std::path::PathBuf::from("."));

    // Handle both absolute and relative paths
    let working_path = std::path::PathBuf::from(working_directory);
    let project_dir = if working_path.is_absolute() {
        // If working_directory is absolute, use it directly
        working_path.clone()
    } else {
        // If relative, join with workspace_dir
        workspace_dir.join(working_directory)
    };

    // For git operations, we need the repository root, not the working directory
    // Try to find the git root by looking for .git directory
    let mut git_root = project_dir.clone();
    let mut found_git = false;
    while git_root.parent().is_some() {
        if git_root.join(".git").exists() {
            found_git = true;
            break;
        }
        if let Some(parent) = git_root.parent() {
            git_root = parent.to_path_buf();
        } else {
            break;
        }
    }

    // If we didn't find a .git directory, fall back to the project directory
    if !found_git {
        git_root.clone_from(&project_dir);
    }

    eprintln!("🔍 Using project directory: {}", project_dir.display());
    eprintln!("🔍 Using git root directory: {}", git_root.display());

    // Change to git root for git commands
    std::env::set_current_dir(&git_root).with_context(|| {
        format!(
            "Failed to navigate to git root directory: {}",
            git_root.display()
        )
    })?;

    // Auto-detect repository URL (fail if not available)
    let repository_url = get_git_remote_url()
        .context("Failed to auto-detect repository URL. Ensure you're in a git repository with origin remote.")?;
    validate_repository_url(&repository_url)?;

    // Handle source branch - use provided value, config default, or auto-detect from git
    let source_branch = arguments
        .get("source_branch")
        .and_then(|v| v.as_str())
        .map_or_else(|| config.defaults.docs.source_branch.clone(), String::from);

    // Check for uncommitted changes and push them before starting docs generation
    eprintln!("🔍 Checking for uncommitted changes...");
    let status_output = Command::new("git")
        .args(["status", "--porcelain"])
        .output()
        .context("Failed to check git status")?;

    if status_output.status.success() {
        let status_text = String::from_utf8(status_output.stdout)?;
        if status_text.trim().is_empty() {
            eprintln!("✅ No uncommitted changes found");
        } else {
            eprintln!("📝 Found uncommitted changes, committing and pushing...");

            // Configure git user for commits (required for git commit to work)
            let config_name_result = Command::new("git")
                .args(["config", "user.name", "MCP Server"])
                .output()
                .context("Failed to configure git user.name")?;

            if !config_name_result.status.success() {
                return Err(anyhow!(
                    "Failed to configure git user.name: {}",
                    String::from_utf8_lossy(&config_name_result.stderr)
                ));
            }

            let config_email_result = Command::new("git")
                .args(["config", "user.email", "mcp-server@5dlabs.com"])
                .output()
                .context("Failed to configure git user.email")?;

            if !config_email_result.status.success() {
                return Err(anyhow!(
                    "Failed to configure git user.email: {}",
                    String::from_utf8_lossy(&config_email_result.stderr)
                ));
            }

            // Add all changes
            let add_result = Command::new("git")
                .args(["add", "."])
                .output()
                .context("Failed to stage changes")?;

            if !add_result.status.success() {
                return Err(anyhow!(
                    "Failed to stage changes: {}",
                    String::from_utf8_lossy(&add_result.stderr)
                ));
            }

            // Commit with timestamp
            let timestamp = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs();
            let commit_msg = format!("docs: auto-commit before docs generation at {timestamp}");

            let commit_result = Command::new("git")
                .args(["commit", "-m", &commit_msg])
                .output()
                .context("Failed to commit changes")?;

            if !commit_result.status.success() {
                let stderr = String::from_utf8_lossy(&commit_result.stderr);
                return Err(anyhow!("Failed to commit changes: {stderr}"));
            }

            // Push to current branch
            let push_result = Command::new("git")
                .args(["push", "origin", &source_branch])
                .output()
                .context("Failed to push changes")?;

            if !push_result.status.success() {
                let stderr = String::from_utf8_lossy(&push_result.stderr);
                return Err(anyhow!("Failed to push changes: {stderr}"));
            }

            eprintln!("✅ Changes committed and pushed successfully");
        }
    } else {
        return Err(anyhow!(
            "Failed to check git status: {}",
            String::from_utf8_lossy(&status_output.stderr)
        ));
    }

    // Handle agent name resolution with validation
    let agent_name = arguments.get("agent").and_then(|v| v.as_str());
    let github_app = if let Some(agent) = agent_name {
        // Validate agent name exists in config
        if !config.agents.contains_key(agent) {
            let available_agents: Vec<&String> = config.agents.keys().collect();
            return Err(anyhow!(
                "Unknown agent '{agent}'. Available agents: {available_agents:?}"
            ));
        }
        config.agents[agent].github_app.clone()
    } else {
        // Use default from config
        config.defaults.docs.github_app.clone()
    };

    // Resolve selected agent key for precedence decisions
    let selected_agent_key: Option<String> = if let Some(agent) = agent_name {
        Some(agent.to_string())
    } else {
        // Find agent whose github_app matches the docs default github app
        config.agents.iter().find_map(|(k, v)| {
            if v.github_app == github_app {
                Some(k.clone())
            } else {
                None
            }
        })
    };

    // Handle model precedence: explicit arg > agent model > docs default model (deprecated)
    let model = if let Some(m) = arguments.get("model").and_then(|v| v.as_str()) {
        m.to_string()
    } else if let Some(agent_key) = &selected_agent_key {
        let agent_model = &config.agents[agent_key].model;
        if agent_model.is_empty() {
            eprintln!(
                "⚠️ INFO: Agent '{agent_key}' has empty model; falling back to defaults.docs.model (deprecated)"
            );
            config.defaults.docs.model.clone()
        } else {
            agent_model.clone()
        }
    } else {
        eprintln!(
            "⚠️ INFO: No agent resolved; using defaults.docs.model (deprecated): {}",
            config.defaults.docs.model
        );
        config.defaults.docs.model.clone()
    };

    // Validate model name (support both Claude API and CLAUDE code formats)
    validate_model_name(&model)?;

    // Task files will be generated by container script from tasks.json

    // Handle include_codebase - use provided value or config default
    let include_codebase = arguments
        .get("include_codebase")
        .and_then(serde_json::Value::as_bool)
        .unwrap_or(config.defaults.docs.include_codebase);

    // Calculate relative working directory for container (relative to git root)
    let container_working_directory = if let Ok(relative_path) = project_dir.strip_prefix(&git_root)
    {
        // Get the relative path from git root to working directory
        relative_path.to_string_lossy().to_string()
    } else if working_path.is_absolute() {
        // Fallback: extract just the final component(s)
        working_path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or(working_directory)
            .to_string()
    } else {
        // If it's already relative, use it as-is
        working_directory.to_string()
    };

    let workflow_model = model.clone();

    let mut params = vec![
        format!("working-directory={container_working_directory}"),
        format!("repository-url={repository_url}"),
        format!("source-branch={source_branch}"),
        format!("github-app={github_app}"),
        format!("model={workflow_model}"),
    ];

    // Always add include_codebase parameter as boolean (required by workflow template)
    params.push(format!("include-codebase={include_codebase}"));

    // New: pass agent-specific tool hints from cto-config.json via workflow params
    // so the controller can merge them server-side into the ConfigMap.
    if let Some(agent_key) = &selected_agent_key {
        if let Some(agent_cfg) = config.agents.get(agent_key) {
            if let Some(tools) = &agent_cfg.tools {
                // remote-tools
                if !tools.remote.is_empty() {
                    let json =
                        serde_json::to_string(&tools.remote).unwrap_or_else(|_| "[]".to_string());
                    params.push(format!("remote-tools={json}"));
                }
                // local-tools: enable servers that are present and marked enabled
                if let Some(ls) = &tools.local_servers {
                    let mut local_list: Vec<&str> = Vec::new();
                    for (name, cfg) in ls {
                        if cfg.enabled {
                            local_list.push(name.as_str());
                        }
                    }
                    if !local_list.is_empty() {
                        let json =
                            serde_json::to_string(&local_list).unwrap_or_else(|_| "[]".to_string());
                        params.push(format!("local-tools={json}"));
                    }
                }
            }
        }
    }

    let mut args = vec![
        "submit",
        "--from",
        "workflowtemplate/docsrun-template",
        "-n",
        "cto",
    ];

    // Add all parameters to the command
    for param in &params {
        args.push("-p");
        args.push(param);
    }

    match run_argo_cli(&args) {
        Ok(output) => Ok(json!({
            "success": true,
            "message": "Documentation generation workflow submitted successfully",
            "output": output,
            "working_directory": working_directory,
            "repository_url": repository_url,
            "source_branch": source_branch,
            "github_app": github_app,
            "agent": agent_name.unwrap_or("default"),
            "model": model,
            "parameters": params
        })),
        Err(e) => Err(anyhow!("Failed to submit docs workflow: {e}")),
    }
}

// ========== Play Progress Tracking Helpers ==========

/// Play workflow status
#[derive(Debug, Clone, PartialEq)]
enum PlayStatus {
    InProgress,
    Suspended,
    Failed,
    Completed,
}

impl std::fmt::Display for PlayStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InProgress => write!(f, "in-progress"),
            Self::Suspended => write!(f, "suspended"),
            Self::Failed => write!(f, "failed"),
            Self::Completed => write!(f, "completed"),
        }
    }
}

/// Play progress data
#[derive(Debug, Clone)]
struct PlayProgress {
    repository: String,
    branch: String,
    current_task_id: Option<u32>,
    workflow_name: Option<String>,
    status: PlayStatus,
    stage: Option<String>,
}

/// Generate `ConfigMap` name from repository
fn configmap_name(repo: &str) -> String {
    format!("play-progress-{}", repo.replace('/', "-"))
}

/// Read play progress from `ConfigMap`
fn read_play_progress(repo: &str) -> Result<Option<PlayProgress>> {
    let name = configmap_name(repo);
    let kubectl_cmd = find_command("kubectl");

    let output = Command::new(&kubectl_cmd)
        .args(["get", "configmap", &name, "-n", "cto", "-o", "json"])
        .output();

    match output {
        Ok(out) if out.status.success() => {
            let cm: Value = serde_json::from_slice(&out.stdout)?;
            let data = cm.get("data");

            if let Some(data_obj) = data.and_then(|d| d.as_object()) {
                let repository = data_obj
                    .get("repository")
                    .and_then(|v| v.as_str())
                    .unwrap_or(repo)
                    .to_string();

                let branch = data_obj
                    .get("branch")
                    .and_then(|v| v.as_str())
                    .unwrap_or("main")
                    .to_string();

                let current_task_id = data_obj
                    .get("current-task-id")
                    .and_then(|v| v.as_str())
                    .and_then(|s| s.parse::<u32>().ok());

                let workflow_name = data_obj
                    .get("workflow-name")
                    .and_then(|v| v.as_str())
                    .map(String::from);

                let status = data_obj
                    .get("status")
                    .and_then(|v| v.as_str())
                    .and_then(|s| match s {
                        "in-progress" => Some(PlayStatus::InProgress),
                        "suspended" => Some(PlayStatus::Suspended),
                        "failed" => Some(PlayStatus::Failed),
                        "completed" => Some(PlayStatus::Completed),
                        _ => None,
                    })
                    .unwrap_or(PlayStatus::InProgress);

                let stage = data_obj
                    .get("stage")
                    .and_then(|v| v.as_str())
                    .map(String::from);

                Ok(Some(PlayProgress {
                    repository,
                    branch,
                    current_task_id,
                    workflow_name,
                    status,
                    stage,
                }))
            } else {
                Ok(None)
            }
        }
        Ok(_) | Err(_) => Ok(None), // ConfigMap doesn't exist or error
    }
}

/// Write play progress to `ConfigMap`
fn write_play_progress(progress: &PlayProgress) -> Result<()> {
    let name = configmap_name(&progress.repository);
    let kubectl_cmd = find_command("kubectl");

    // Build ConfigMap JSON
    let mut data = serde_json::Map::new();
    data.insert("repository".to_string(), json!(progress.repository));
    data.insert("branch".to_string(), json!(progress.branch));

    if let Some(task_id) = progress.current_task_id {
        data.insert("current-task-id".to_string(), json!(task_id.to_string()));
    }

    if let Some(ref workflow_name) = progress.workflow_name {
        data.insert("workflow-name".to_string(), json!(workflow_name));
    }

    data.insert("status".to_string(), json!(progress.status.to_string()));

    if let Some(ref stage) = progress.stage {
        data.insert("stage".to_string(), json!(stage));
    }

    let now = chrono::Utc::now().to_rfc3339();
    data.insert("last-updated".to_string(), json!(now));

    let cm = json!({
        "apiVersion": "v1",
        "kind": "ConfigMap",
        "metadata": {
            "name": name,
            "namespace": "cto",
            "labels": {
                "play-tracking": "true"
            }
        },
        "data": data
    });

    // Try to create or update
    let cm_json = serde_json::to_string(&cm)?;
    let mut temp_file = NamedTempFile::new()?;
    temp_file.write_all(cm_json.as_bytes())?;
    temp_file.flush()?;

    let result = Command::new(&kubectl_cmd)
        .args(["apply", "-f", temp_file.path().to_str().unwrap()])
        .output();

    match result {
        Ok(out) if out.status.success() => Ok(()),
        Ok(out) => Err(anyhow!(
            "Failed to write ConfigMap: {}",
            String::from_utf8_lossy(&out.stderr)
        )),
        Err(e) => Err(anyhow!("Failed to execute kubectl: {e}")),
    }
}

/// Clear play progress `ConfigMap`
fn clear_play_progress(repo: &str) {
    let name = configmap_name(repo);
    let kubectl_cmd = find_command("kubectl");

    let _ = Command::new(&kubectl_cmd)
        .args([
            "delete",
            "configmap",
            &name,
            "-n",
            "cto",
            "--ignore-not-found=true",
        ])
        .output();
}

/// Query active play workflows for a repository
fn find_active_play_workflow(repo: &str) -> Result<Option<(String, u32, String)>> {
    let argo_cmd = find_command("argo");

    // Query workflows with play labels
    let output = Command::new(&argo_cmd)
        .args([
            "list",
            "-n",
            "cto",
            "-l",
            &format!("repository={}", repo.replace('/', "-")),
            "-l",
            "workflow-type=play",
            "-o",
            "json",
        ])
        .output()?;

    if !output.status.success() {
        return Ok(None);
    }

    let workflows: Vec<Value> = serde_json::from_slice(&output.stdout).unwrap_or_default();

    // Find a running or suspended workflow
    for wf in workflows {
        if let Some(status) = wf
            .get("status")
            .and_then(|s| s.get("phase"))
            .and_then(|p| p.as_str())
        {
            if status == "Running" || status == "Suspended" {
                let workflow_name = wf
                    .get("metadata")
                    .and_then(|m| m.get("name"))
                    .and_then(|n| n.as_str())
                    .unwrap_or("")
                    .to_string();

                let task_id = wf
                    .get("metadata")
                    .and_then(|m| m.get("labels"))
                    .and_then(|l| l.get("task-id"))
                    .and_then(|t| t.as_str())
                    .and_then(|s| s.parse::<u32>().ok())
                    .unwrap_or(0);

                let phase = status.to_string();

                if !workflow_name.is_empty() && task_id > 0 {
                    return Ok(Some((workflow_name, task_id, phase)));
                }
            }
        }
    }

    Ok(None)
}

// ========== TaskMaster Integration Helpers ==========

#[derive(Debug, Clone, Deserialize)]
struct TaskMasterTask {
    id: u32,
    title: String,
    status: String,
    #[serde(default)]
    priority: Option<String>,
    #[serde(default)]
    dependencies: Option<Vec<u32>>,
}

#[derive(Debug, Deserialize)]
struct TaskTag {
    tasks: Vec<TaskMasterTask>,
}

#[derive(Debug, Deserialize)]
#[serde(untagged)]
enum TasksFile {
    /// New tagged format: { "master": { "tasks": [...] }, "other-tag": { "tasks": [...] } }
    Tagged(HashMap<String, TaskTag>),
    /// Legacy flat format: { "tasks": [...] }
    Flat { tasks: Vec<TaskMasterTask> },
}

/// Find tasks.json in repository, optionally starting from a working directory
fn find_tasks_file(working_dir: Option<&str>) -> Option<std::path::PathBuf> {
    // If working_dir is an absolute path, use it directly
    let (base_dir, workspace_dir) = if let Some(wd) = working_dir {
        let wd_path = std::path::PathBuf::from(wd);
        if wd_path.is_absolute() {
            (wd_path, None)
        } else {
            // Try to get workspace directory for relative paths
            let ws_dir = resolve_workspace_dir()?;
            let base = ws_dir.join(wd);
            (base, Some(ws_dir))
        }
    } else {
        // Try to get workspace directory
        let ws_dir = resolve_workspace_dir()?;
        (ws_dir.clone(), Some(ws_dir))
    };

    let mut candidates = vec![
        base_dir
            .join(".taskmaster")
            .join("tasks")
            .join("tasks.json"),
        base_dir.join(".taskmaster").join("tasks.json"),
        base_dir.join("tasks.json"),
    ];

    // If working_dir was provided and we have a workspace, also try workspace root as fallback
    if working_dir.is_some() {
        if let Some(ws_dir) = workspace_dir {
            candidates.push(ws_dir.join(".taskmaster").join("tasks").join("tasks.json"));
            candidates.push(ws_dir.join(".taskmaster").join("tasks.json"));
            candidates.push(ws_dir.join("tasks.json"));
        }
    }

    candidates.into_iter().find(|p| p.exists())
}

/// Get next available task from `TaskMaster`
fn get_next_taskmaster_task(working_dir: Option<&str>) -> Result<Option<TaskMasterTask>> {
    let tasks_file =
        find_tasks_file(working_dir).ok_or_else(|| anyhow!("tasks.json not found in workspace"))?;

    let content = std::fs::read_to_string(&tasks_file)
        .with_context(|| format!("Failed to read tasks file: {}", tasks_file.display()))?;

    let tasks_data: TasksFile = serde_json::from_str(&content)
        .with_context(|| format!("Failed to parse tasks.json: {}", tasks_file.display()))?;

    // Extract tasks from either format
    let tasks = match tasks_data {
        TasksFile::Tagged(mut tags) => {
            // Use "master" tag by default, or first available tag
            tags.remove("master")
                .or_else(|| tags.into_values().next())
                .ok_or_else(|| anyhow!("No task tags found in tasks.json"))?
                .tasks
        }
        TasksFile::Flat { tasks } => tasks,
    };

    // Build task map for dependency checking
    let task_map: HashMap<u32, &TaskMasterTask> = tasks.iter().map(|t| (t.id, t)).collect();

    // Filter available tasks (not done, all deps satisfied)
    let mut available_tasks: Vec<&TaskMasterTask> = tasks
        .iter()
        .filter(|task| {
            // Skip done tasks
            if task.status == "done" || task.status == "completed" {
                return false;
            }

            // Check dependencies
            if let Some(deps) = &task.dependencies {
                for dep_id in deps {
                    if let Some(dep_task) = task_map.get(dep_id) {
                        if dep_task.status != "done" && dep_task.status != "completed" {
                            return false;
                        }
                    } else {
                        return false; // Non-existent dependency
                    }
                }
            }

            true
        })
        .collect();

    if available_tasks.is_empty() {
        return Ok(None);
    }

    // Sort by priority (high > medium > low), then by ID
    available_tasks.sort_by(|a, b| {
        let priority_order = |p: &Option<String>| match p.as_deref() {
            Some("high") => 0,
            Some("low") => 2,
            _ => 1, // medium or unspecified
        };

        let a_priority = priority_order(&a.priority);
        let b_priority = priority_order(&b.priority);

        match a_priority.cmp(&b_priority) {
            std::cmp::Ordering::Equal => a.id.cmp(&b.id),
            other => other,
        }
    });

    Ok(available_tasks.first().map(|&t| t.clone()))
}

/// Find blocked tasks (tasks with all pending dependencies)
fn find_blocked_taskmaster_tasks(working_dir: Option<&str>) -> Result<Vec<TaskMasterTask>> {
    let tasks_file =
        find_tasks_file(working_dir).ok_or_else(|| anyhow!("tasks.json not found in workspace"))?;

    let content = std::fs::read_to_string(&tasks_file)
        .with_context(|| format!("Failed to read tasks file: {}", tasks_file.display()))?;

    let tasks_data: TasksFile = serde_json::from_str(&content)
        .with_context(|| format!("Failed to parse tasks.json: {}", tasks_file.display()))?;

    // Extract tasks from either format
    let tasks = match tasks_data {
        TasksFile::Tagged(mut tags) => {
            // Use "master" tag by default, or first available tag
            tags.remove("master")
                .or_else(|| tags.into_values().next())
                .ok_or_else(|| anyhow!("No task tags found in tasks.json"))?
                .tasks
        }
        TasksFile::Flat { tasks } => tasks,
    };
    let task_map: HashMap<u32, &TaskMasterTask> = tasks.iter().map(|t| (t.id, t)).collect();

    let mut blocked = Vec::new();

    for task in &tasks {
        // Skip done tasks
        if task.status == "done" || task.status == "completed" {
            continue;
        }

        // Check if this task has dependencies
        if let Some(deps) = &task.dependencies {
            if deps.is_empty() {
                continue;
            }

            // Check if ALL dependencies are still pending/in-progress
            let all_deps_blocked = deps.iter().all(|dep_id| {
                task_map
                    .get(dep_id)
                    .is_none_or(|dep| dep.status != "done" && dep.status != "completed")
            });

            if all_deps_blocked {
                blocked.push(task.clone());
            }
        }
    }

    Ok(blocked)
}

/// Handle play status query
#[allow(clippy::disallowed_macros)]
fn handle_play_status(arguments: &HashMap<String, Value>) -> Result<Value> {
    let config = CTO_CONFIG.get().unwrap();

    // Handle repository - use provided value or config default
    let repository = arguments
        .get("repository")
        .and_then(|v| v.as_str())
        .map(String::from)
        .or_else(|| config.defaults.play.repository.clone())
        .ok_or(anyhow!("No repository specified. Please provide a 'repository' parameter or set defaults.play.repository in config"))?;

    // Get docs project directory for finding tasks.json
    // Note: We use docs_project_directory (where tasks live), NOT working_directory (where code lives)
    let docs_dir = config
        .defaults
        .play
        .docs_project_directory
        .as_ref()
        .and_then(|dd| if dd == "." { None } else { Some(dd.as_str()) });

    // Read progress from ConfigMap
    let progress = read_play_progress(&repository)?;

    // Check for active workflow in Argo
    let active_workflow = find_active_play_workflow(&repository)?;

    // Check for blocked tasks
    let blocked_tasks = find_blocked_taskmaster_tasks(docs_dir).unwrap_or_default();

    // Build comprehensive status response
    match (progress, active_workflow) {
        (Some(prog), Some((wf_name, wf_task, wf_phase))) => {
            // Active workflow found
            Ok(json!({
                "success": true,
                "status": "active",
                "repository": repository,
                "current_task_id": wf_task,
                "workflow_name": wf_name,
                "workflow_phase": wf_phase,
                "stage": prog.stage,
                "configmap_status": prog.status.to_string(),
                "argo_url": format!("https://argo.5dlabs.com/workflows/cto/{}", wf_name),
            }))
        }
        (Some(prog), None) => {
            // ConfigMap exists but no active workflow - orphaned
            eprintln!("⚠️  Orphaned ConfigMap detected for {repository}");
            Ok(json!({
                "success": true,
                "status": "orphaned",
                "repository": repository,
                "last_task_id": prog.current_task_id,
                "last_workflow_name": prog.workflow_name,
                "message": "ConfigMap exists but workflow not found. ConfigMap will be cleared on next play submission.",
            }))
        }
        (None, Some((wf_name, wf_task, wf_phase))) => {
            // Workflow exists but no ConfigMap - legacy or external workflow
            Ok(json!({
                "success": true,
                "status": "active_legacy",
                "repository": repository,
                "current_task_id": wf_task,
                "workflow_name": wf_name,
                "workflow_phase": wf_phase,
                "message": "Workflow active but no progress tracking (legacy workflow)",
                "argo_url": format!("https://argo.5dlabs.com/workflows/cto/{}", wf_name),
            }))
        }
        (None, None) => {
            // No active workflow

            // Try to get next task (only works if repository is in local workspace)
            let (next_task, tasks_found) = match get_next_taskmaster_task(docs_dir) {
                Ok(task) => (task, true),
                Err(err) => {
                    eprintln!("ℹ️  Unable to read TaskMaster tasks locally: {err}");
                    (None, false)
                }
            };

            if let Some(task) = next_task {
                Ok(json!({
                    "success": true,
                    "status": "idle",
                    "repository": repository,
                    "message": "No active workflow",
                    "next_available_task": {
                        "id": task.id,
                        "title": task.title,
                        "priority": task.priority,
                    },
                }))
            } else {
                // Determine appropriate message based on whether tasks.json was found
                let message = if !tasks_found {
                    // tasks.json not found - repository likely not in workspace
                    "No active workflow. Repository may not be in local workspace - use task_id to start a new workflow.".to_string()
                } else if blocked_tasks.is_empty() {
                    // tasks.json found but no tasks available
                    "All tasks completed".to_string()
                } else {
                    // tasks.json found but tasks are blocked
                    format!("{} task(s) blocked by dependencies", blocked_tasks.len())
                };

                Ok(json!({
                    "success": true,
                    "status": "idle",
                    "repository": repository,
                    "message": message,
                    "blocked_tasks": blocked_tasks.into_iter().map(|t| json!({
                        "id": t.id,
                        "title": t.title,
                        "dependencies": t.dependencies
                    })).collect::<Vec<_>>(),
                }))
            }
        }
    }
}

#[allow(clippy::disallowed_macros, clippy::too_many_lines)]
fn handle_play_workflow(arguments: &HashMap<String, Value>) -> Result<Value> {
    use base64::{engine::general_purpose, Engine as _};

    let config = CTO_CONFIG.get().unwrap();

    // Handle repository - use provided value or config default
    let repository = arguments
        .get("repository")
        .and_then(|v| v.as_str())
        .map(String::from)
        .or_else(|| config.defaults.play.repository.clone())
        .ok_or(anyhow!("No repository specified. Please provide a 'repository' parameter or set defaults.play.repository in config"))?;

    // Validate repository URL
    validate_repository_url(&repository)?;

    // Check for explicit repository_path parameter
    let repository_path = arguments
        .get("repository_path")
        .and_then(|v| v.as_str())
        .map(String::from);

    // Get docs project directory for finding tasks.json
    // Note: We use docs_project_directory (where tasks live), NOT working_directory (where code lives)
    let docs_dir = if let Some(repo_path) = &repository_path {
        // If repository_path is provided, construct full path to docs directory
        eprintln!("📁 Using explicit repository path: {repo_path}");
        let docs_project_dir = config
            .defaults
            .play
            .docs_project_directory
            .as_deref()
            .unwrap_or("docs");

        let full_docs_path = if docs_project_dir == "." {
            repo_path.clone()
        } else {
            format!("{repo_path}/{docs_project_dir}")
        };

        eprintln!("   Looking for tasks in: {full_docs_path}");
        Some(full_docs_path)
    } else {
        // Otherwise use config with workspace detection
        config
            .defaults
            .play
            .docs_project_directory
            .as_ref()
            .and_then(|dd| if dd == "." { None } else { Some(dd.clone()) })
    };

    // Check if task_id is provided
    let task_id = if let Some(id_value) = arguments.get("task_id") {
        // Explicit task_id provided
        #[allow(clippy::cast_possible_truncation)]
        Some(
            id_value
                .as_u64()
                .ok_or(anyhow!("Invalid task_id parameter"))? as u32,
        )
    } else {
        // Auto-detection mode
        eprintln!("🔍 Auto-detecting next task (no task_id provided)...");

        // 1. Check ConfigMap for current progress
        if let Some(progress) = read_play_progress(&repository)? {
            eprintln!("📋 Found existing progress for {repository}");

            // 2. Validate against Argo
            if let Some(workflow_name) = &progress.workflow_name {
                if let Some((active_wf, active_task, phase)) =
                    find_active_play_workflow(&repository)?
                {
                    if active_wf == *workflow_name {
                        // Workflow still active
                        return Ok(json!({
                            "success": false,
                            "message": format!(
                                "Play workflow already active for {}: task {} (phase: {})",
                                repository, active_task, phase
                            ),
                            "workflow_name": active_wf,
                            "task_id": active_task,
                            "phase": phase,
                            "status": progress.status.to_string(),
                        }));
                    }
                }

                // Workflow not found but ConfigMap exists - orphaned state
                eprintln!(
                    "⚠️  Orphaned progress detected: ConfigMap exists but workflow not found"
                );
                clear_play_progress(&repository);
            }
        }

        // 3. If repository_path is provided, skip workspace detection and use it directly
        if repository_path.is_some() {
            eprintln!("🔍 Using explicit repository path - querying TaskMaster...");
            match get_next_taskmaster_task(docs_dir.as_deref()) {
                Ok(Some(task)) => {
                    eprintln!("✅ Found next task: {} - {}", task.id, task.title);
                    Some(task.id)
                }
                Ok(None) => {
                    // Check for blocked tasks to provide helpful feedback
                    let blocked_tasks =
                        find_blocked_taskmaster_tasks(docs_dir.as_deref()).unwrap_or_default();

                    let message = if blocked_tasks.is_empty() {
                        "No tasks available - all tasks are completed".to_string()
                    } else {
                        let blocked_ids: Vec<String> = blocked_tasks
                            .iter()
                            .map(|t| format!("Task {} ({})", t.id, t.title))
                            .collect();

                        format!(
                            "No tasks available. {} task(s) blocked by dependencies:\n{}",
                            blocked_tasks.len(),
                            blocked_ids.join("\n")
                        )
                    };

                    return Ok(json!({
                        "success": false,
                        "message": message,
                        "repository": repository,
                        "blocked_tasks": blocked_tasks.into_iter().map(|t| json!({
                            "id": t.id,
                            "title": t.title,
                            "dependencies": t.dependencies
                        })).collect::<Vec<_>>(),
                    }));
                }
                Err(e) => {
                    // Missing tasks.json is a serious error when repository_path is explicitly provided
                    eprintln!("❌ Could not find tasks.json at repository_path: {e}");
                    return Err(anyhow!(
                        "tasks.json not found at specified repository_path: {}. Please ensure .taskmaster/tasks/tasks.json exists.",
                        repository_path.as_ref().unwrap()
                    ));
                }
            }
        } else {
            // Check if repository is in local workspace
            // Get current workspace repository (if available)
            let workspace_repo = resolve_workspace_dir().and_then(|workspace_path| {
                let output = std::process::Command::new("git")
                    .args(["remote", "get-url", "origin"])
                    .current_dir(&workspace_path)
                    .output()
                    .ok()?;

                if output.status.success() {
                    let url = String::from_utf8_lossy(&output.stdout).trim().to_string();
                    url.strip_prefix("git@github.com:")
                        .or_else(|| url.strip_prefix("https://github.com/"))
                        .map(|repo_part| repo_part.trim_end_matches(".git").to_string())
                } else {
                    None
                }
            });

            // Normalize repository to org/repo format for comparison
            let normalized_repo = if repository.starts_with("https://github.com/") {
                repository
                    .strip_prefix("https://github.com/")
                    .unwrap()
                    .trim_end_matches(".git")
                    .to_string()
            } else {
                repository.clone()
            };

            // Check if the requested repository matches the workspace
            let is_local_repo = workspace_repo.as_ref() == Some(&normalized_repo);

            if is_local_repo {
                // Repository is local - try to auto-detect next task
                eprintln!("🔍 Querying TaskMaster for next available task...");
                match get_next_taskmaster_task(docs_dir.as_deref()) {
                    Ok(Some(task)) => {
                        eprintln!("✅ Found next task: {} - {}", task.id, task.title);
                        Some(task.id)
                    }
                    Ok(None) => {
                        // Check for blocked tasks to provide helpful feedback
                        let blocked_tasks =
                            find_blocked_taskmaster_tasks(docs_dir.as_deref()).unwrap_or_default();

                        let message = if blocked_tasks.is_empty() {
                            "No tasks available - all tasks are completed".to_string()
                        } else {
                            let blocked_ids: Vec<String> = blocked_tasks
                                .iter()
                                .map(|t| format!("Task {} ({})", t.id, t.title))
                                .collect();

                            format!(
                                "No tasks available. {} task(s) blocked by dependencies:\n{}",
                                blocked_tasks.len(),
                                blocked_ids.join("\n")
                            )
                        };

                        return Ok(json!({
                            "success": false,
                            "message": message,
                            "repository": repository,
                            "blocked_tasks": blocked_tasks.into_iter().map(|t| json!({
                                "id": t.id,
                                "title": t.title,
                                "dependencies": t.dependencies
                            })).collect::<Vec<_>>(),
                        }));
                    }
                    Err(e) => {
                        // Unexpected error reading tasks.json
                        eprintln!("❌ Error reading tasks.json: {e}");
                        return Err(anyhow!("Failed to read tasks.json: {e}"));
                    }
                }
            } else {
                // Repository is not in local workspace
                eprintln!("📦 Repository '{repository}' is not in local workspace");
                eprintln!(
                    "   Workspace repository: {}",
                    workspace_repo.as_deref().unwrap_or("unknown")
                );
                eprintln!("⚠️  Cannot auto-detect tasks for remote repository");
                eprintln!("   Please specify task_id explicitly or use repository_path parameter");

                return Ok(json!({
                    "success": false,
                    "message": "Repository not in local workspace. Please specify task_id explicitly or use repository_path parameter to point to the local repository location.",
                    "repository": repository,
                    "hint": "Use cto_play({ task_id: 1 }) or cto_play({ repository_path: '/path/to/repo' })"
                }));
            }
        }
    };

    let task_id = task_id.ok_or(anyhow!("Failed to determine task_id"))?;

    // Handle service - use provided value or config default
    let service = arguments
        .get("service")
        .and_then(|v| v.as_str())
        .or(config.defaults.play.service.as_deref())
        .ok_or(anyhow!("Missing required parameter: service. Please provide it or set defaults.play.service in config"))?;

    // Validate service name (must be valid for PVC naming)
    if !service
        .chars()
        .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-')
    {
        return Err(anyhow!(
            "Invalid service name '{service}'. Must contain only lowercase letters, numbers, and hyphens"
        ));
    }

    // Handle docs repository - use provided value, config default, or error
    let docs_repository = arguments.get("docs_repository")
        .and_then(|v| v.as_str())
        .map(String::from)
        .or_else(|| config.defaults.play.docs_repository.clone())
        .ok_or(anyhow!("No docs_repository specified. Please provide a 'docs_repository' parameter or set defaults.play.docsRepository in config"))?;

    validate_repository_url(&docs_repository)?;

    // Handle docs project directory - use provided value or config default
    let docs_project_directory = arguments
        .get("docs_project_directory")
        .and_then(|v| v.as_str())
        .or(config.defaults.play.docs_project_directory.as_deref())
        .ok_or(anyhow!("Missing required parameter: docs_project_directory. Please provide it or set defaults.play.docsProjectDirectory in config"))?;

    // Handle CLI - use provided value or config default (needed for agent resolution)
    let cli = arguments
        .get("cli")
        .and_then(|v| v.as_str())
        .map_or_else(|| config.defaults.play.cli.clone(), String::from);

    // Handle model - use provided value or config default (needed for agent resolution)
    let model = arguments
        .get("model")
        .and_then(|v| v.as_str())
        .map_or_else(|| config.defaults.play.model.clone(), String::from);

    // Try to load repository-specific configuration for agent tools
    eprintln!("🔍 Checking for repository-specific configuration...");
    let repo_config = load_repository_config(repository_path.as_deref());

    // Use repository config if available, otherwise fall back to platform config
    let effective_config = repo_config.as_ref().unwrap_or(config);

    if repo_config.is_some() {
        eprintln!("✅ Using repository configuration for agent tools");
    } else {
        eprintln!("ℹ️  Using platform configuration (no repository config)");
    }

    // Handle implementation agent - use provided value or config default
    let implementation_agent_input = arguments
        .get("implementation_agent")
        .and_then(|v| v.as_str())
        .map_or_else(
            || effective_config.defaults.play.implementation_agent.clone(),
            String::from,
        );

    let implementation_agent_cfg = effective_config
        .agents
        .values()
        .find(|a| a.github_app == implementation_agent_input);

    // Resolve agent name and extract CLI/model/tools/modelRotation if it's a short alias
    let (
        implementation_agent,
        implementation_cli,
        implementation_model,
        implementation_tools,
        implementation_model_rotation,
    ) = if let Some(agent_config) = implementation_agent_cfg {
        // Use the structured agent configuration
        let agent_cli = if agent_config.cli.is_empty() {
            cli.clone()
        } else {
            agent_config.cli.clone()
        };
        let agent_model = if agent_config.model.is_empty() {
            model.clone()
        } else {
            agent_config.model.clone()
        };
        let agent_tools = agent_config
            .tools
            .as_ref()
            .map(|t| match serde_json::to_string(t) {
                Ok(json) => {
                    eprintln!("✅ Serialized implementation agent tools: {json}");
                    json
                }
                Err(e) => {
                    eprintln!("❌ Failed to serialize implementation agent tools: {e}");
                    eprintln!("   Tools data: {t:?}");
                    "{}".to_string()
                }
            })
            .unwrap_or_else(|| {
                eprintln!(
                    "ℹ️ No tools configured for implementation agent {implementation_agent_input}"
                );
                "{}".to_string()
            });
        let agent_model_rotation = agent_config
            .model_rotation
            .as_ref()
            .and_then(|mr| {
                if mr.enabled && !mr.models.is_empty() {
                    match serde_json::to_string(&mr.models) {
                        Ok(json) => {
                            eprintln!("✅ Model rotation enabled for implementation agent: {json}");
                            Some(json)
                        }
                        Err(e) => {
                            eprintln!("❌ Failed to serialize model rotation: {e}");
                            None
                        }
                    }
                } else {
                    None
                }
            })
            .unwrap_or_else(|| "[]".to_string());
        (
            agent_config.github_app.clone(),
            agent_cli,
            agent_model,
            agent_tools,
            agent_model_rotation,
        )
    } else {
        // Not a configured agent, use provided name with defaults
        eprintln!("⚠️ Agent {implementation_agent_input} not found in config, using defaults");
        (
            implementation_agent_input.clone(),
            cli.clone(),
            model.clone(),
            "{}".to_string(),
            "[]".to_string(),
        )
    };

    let implementation_agent_max_retries = implementation_agent_cfg.and_then(|cfg| cfg.max_retries);

    // Handle frontend agent - use provided value or config default
    let frontend_agent_input = arguments
        .get("frontend_agent")
        .and_then(|v| v.as_str())
        .map(String::from)
        .or_else(|| effective_config.defaults.play.frontend_agent.clone())
        .unwrap_or_else(|| {
            let fallback = effective_config.defaults.play.implementation_agent.clone();
            eprintln!(
                "⚠️ WARNING: No frontend-agent specified and no defaults.play.frontendAgent in config!"
            );
            eprintln!(
                "   Falling back to implementation-agent: {fallback}"
            );
            eprintln!(
                "   ⚠️ This may cause frontend tasks to be routed incorrectly!"
            );
            eprintln!(
                "   💡 Set defaults.play.frontendAgent in cto-config.json to avoid this"
            );
            fallback
        });

    let frontend_agent_cfg = effective_config
        .agents
        .values()
        .find(|a| a.github_app == frontend_agent_input);

    // Resolve frontend agent name and extract CLI/model/tools/modelRotation
    let (frontend_agent, frontend_cli, frontend_model, frontend_tools, frontend_model_rotation) =
        if let Some(agent_config) = frontend_agent_cfg {
            let agent_cli = if agent_config.cli.is_empty() {
                cli.clone()
            } else {
                agent_config.cli.clone()
            };
            let agent_model = if agent_config.model.is_empty() {
                model.clone()
            } else {
                agent_config.model.clone()
            };
            let agent_tools = agent_config
                .tools
                .as_ref()
                .map(|t| match serde_json::to_string(t) {
                    Ok(json) => {
                        eprintln!("✅ Serialized frontend agent tools: {json}");
                        json
                    }
                    Err(e) => {
                        eprintln!("❌ Failed to serialize frontend agent tools: {e}");
                        eprintln!("   Tools data: {t:?}");
                        "{}".to_string()
                    }
                })
                .unwrap_or_else(|| {
                    eprintln!("ℹ️ No tools configured for frontend agent {frontend_agent_input}");
                    "{}".to_string()
                });
            let agent_model_rotation = agent_config
                .model_rotation
                .as_ref()
                .and_then(|mr| {
                    if mr.enabled && !mr.models.is_empty() {
                        match serde_json::to_string(&mr.models) {
                            Ok(json) => {
                                eprintln!("✅ Model rotation enabled for frontend agent: {json}");
                                Some(json)
                            }
                            Err(e) => {
                                eprintln!("❌ Failed to serialize model rotation: {e}");
                                None
                            }
                        }
                    } else {
                        None
                    }
                })
                .unwrap_or_else(|| "[]".to_string());
            (
                agent_config.github_app.clone(),
                agent_cli,
                agent_model,
                agent_tools,
                agent_model_rotation,
            )
        } else {
            // Not a configured agent, use provided name with defaults
            eprintln!(
                "⚠️ Frontend agent {frontend_agent_input} not found in config, using defaults"
            );
            (
                frontend_agent_input.clone(),
                cli.clone(),
                model.clone(),
                "{}".to_string(),
                "[]".to_string(),
            )
        };

    // Handle quality agent - use provided value or config default
    let quality_agent_input = arguments
        .get("quality_agent")
        .and_then(|v| v.as_str())
        .map_or_else(
            || effective_config.defaults.play.quality_agent.clone(),
            String::from,
        );

    // Resolve agent name and extract CLI/model/tools/modelRotation if it's a short alias
    let quality_agent_cfg = effective_config
        .agents
        .values()
        .find(|a| a.github_app == quality_agent_input);

    let (quality_agent, quality_cli, quality_model, quality_tools, quality_model_rotation) =
        if let Some(agent_config) = quality_agent_cfg {
            // Use the structured agent configuration
            let agent_cli = if agent_config.cli.is_empty() {
                cli.clone()
            } else {
                agent_config.cli.clone()
            };
            let agent_model = if agent_config.model.is_empty() {
                model.clone()
            } else {
                agent_config.model.clone()
            };
            let agent_tools = agent_config
                .tools
                .as_ref()
                .map(|t| match serde_json::to_string(t) {
                    Ok(json) => {
                        eprintln!("✅ Serialized quality agent tools: {json}");
                        json
                    }
                    Err(e) => {
                        eprintln!("❌ Failed to serialize quality agent tools: {e}");
                        eprintln!("   Tools data: {t:?}");
                        "{}".to_string()
                    }
                })
                .unwrap_or_else(|| {
                    eprintln!("ℹ️ No tools configured for quality agent {quality_agent_input}");
                    "{}".to_string()
                });
            let agent_model_rotation = agent_config
                .model_rotation
                .as_ref()
                .and_then(|mr| {
                    if mr.enabled && !mr.models.is_empty() {
                        match serde_json::to_string(&mr.models) {
                            Ok(json) => {
                                eprintln!("✅ Model rotation enabled for quality agent: {json}");
                                Some(json)
                            }
                            Err(e) => {
                                eprintln!("❌ Failed to serialize model rotation: {e}");
                                None
                            }
                        }
                    } else {
                        None
                    }
                })
                .unwrap_or_else(|| "[]".to_string());
            (
                agent_config.github_app.clone(),
                agent_cli,
                agent_model,
                agent_tools,
                agent_model_rotation,
            )
        } else {
            // Not a configured agent, use provided name with defaults
            eprintln!("⚠️ Agent {quality_agent_input} not found in config, using defaults");
            (
                quality_agent_input.clone(),
                cli.clone(),
                model.clone(),
                "{}".to_string(),
                "[]".to_string(),
            )
        };

    let quality_agent_max_retries = quality_agent_cfg.and_then(|cfg| cfg.max_retries);

    // Handle security agent - use provided value or config default
    let security_agent_input = arguments
        .get("security_agent")
        .and_then(|v| v.as_str())
        .map_or_else(
            || effective_config.defaults.play.security_agent.clone(),
            String::from,
        );

    // Resolve agent name and extract CLI/model/tools/modelRotation if it's a short alias
    let security_agent_cfg = effective_config
        .agents
        .values()
        .find(|a| a.github_app == security_agent_input);

    let (security_agent, security_cli, security_model, security_tools, security_model_rotation) =
        if let Some(agent_config) = security_agent_cfg {
            // Use the structured agent configuration
            let agent_cli = if agent_config.cli.is_empty() {
                cli.clone()
            } else {
                agent_config.cli.clone()
            };
            let agent_model = if agent_config.model.is_empty() {
                model.clone()
            } else {
                agent_config.model.clone()
            };
            let agent_tools = agent_config
                .tools
                .as_ref()
                .map(|t| match serde_json::to_string(t) {
                    Ok(json) => {
                        eprintln!("✅ Serialized security agent tools: {json}");
                        json
                    }
                    Err(e) => {
                        eprintln!("❌ Failed to serialize security agent tools: {e}");
                        eprintln!("   Tools data: {t:?}");
                        "{}".to_string()
                    }
                })
                .unwrap_or_else(|| {
                    eprintln!("ℹ️ No tools configured for security agent {security_agent_input}");
                    "{}".to_string()
                });
            let agent_model_rotation = agent_config
                .model_rotation
                .as_ref()
                .and_then(|mr| {
                    if mr.enabled && !mr.models.is_empty() {
                        match serde_json::to_string(&mr.models) {
                            Ok(json) => {
                                eprintln!("✅ Model rotation enabled for security agent: {json}");
                                Some(json)
                            }
                            Err(e) => {
                                eprintln!("❌ Failed to serialize model rotation: {e}");
                                None
                            }
                        }
                    } else {
                        None
                    }
                })
                .unwrap_or_else(|| "[]".to_string());
            (
                agent_config.github_app.clone(),
                agent_cli,
                agent_model,
                agent_tools,
                agent_model_rotation,
            )
        } else {
            // Not a configured agent, use provided name with defaults
            eprintln!("⚠️ Agent {security_agent_input} not found in config, using defaults");
            (
                security_agent_input.clone(),
                cli.clone(),
                model.clone(),
                "{}".to_string(),
                "[]".to_string(),
            )
        };

    let security_agent_max_retries = security_agent_cfg.and_then(|cfg| cfg.max_retries);

    // Handle testing agent - use provided value or config default
    let testing_agent_input = arguments
        .get("testing_agent")
        .and_then(|v| v.as_str())
        .map_or_else(
            || effective_config.defaults.play.testing_agent.clone(),
            String::from,
        );

    // Resolve agent name and extract CLI/model/tools/modelRotation if it's a short alias
    let testing_agent_cfg = effective_config
        .agents
        .values()
        .find(|a| a.github_app == testing_agent_input);

    let (testing_agent, testing_cli, testing_model, testing_tools, testing_model_rotation) =
        if let Some(agent_config) = testing_agent_cfg {
            // Use the structured agent configuration
            let agent_cli = if agent_config.cli.is_empty() {
                cli.clone()
            } else {
                agent_config.cli.clone()
            };
            let agent_model = if agent_config.model.is_empty() {
                model.clone()
            } else {
                agent_config.model.clone()
            };
            let agent_tools = agent_config
                .tools
                .as_ref()
                .map(|t| match serde_json::to_string(t) {
                    Ok(json) => {
                        eprintln!("✅ Serialized testing agent tools: {json}");
                        json
                    }
                    Err(e) => {
                        eprintln!("❌ Failed to serialize testing agent tools: {e}");
                        eprintln!("   Tools data: {t:?}");
                        "{}".to_string()
                    }
                })
                .unwrap_or_else(|| {
                    eprintln!("ℹ️ No tools configured for testing agent {testing_agent_input}");
                    "{}".to_string()
                });
            let agent_model_rotation = agent_config
                .model_rotation
                .as_ref()
                .and_then(|mr| {
                    if mr.enabled && !mr.models.is_empty() {
                        match serde_json::to_string(&mr.models) {
                            Ok(json) => {
                                eprintln!("✅ Model rotation enabled for testing agent: {json}");
                                Some(json)
                            }
                            Err(e) => {
                                eprintln!("❌ Failed to serialize model rotation: {e}");
                                None
                            }
                        }
                    } else {
                        None
                    }
                })
                .unwrap_or_else(|| "[]".to_string());
            (
                agent_config.github_app.clone(),
                agent_cli,
                agent_model,
                agent_tools,
                agent_model_rotation,
            )
        } else {
            // Not a configured agent, use provided name with defaults
            eprintln!("⚠️ Agent {testing_agent_input} not found in config, using defaults");
            (
                testing_agent_input.clone(),
                cli.clone(),
                model.clone(),
                "{}".to_string(),
                "[]".to_string(),
            )
        };

    let testing_agent_max_retries = testing_agent_cfg.and_then(|cfg| cfg.max_retries);

    let frontend_agent_max_retries = frontend_agent_cfg.and_then(|cfg| cfg.max_retries);

    // Validate model name (support both Claude API and CLAUDE code formats)
    validate_model_name(&model)?;

    // Validate agent-specific models
    validate_model_name(&implementation_model)
        .map_err(|e| anyhow!("Invalid implementation agent model: {e}"))?;
    validate_model_name(&frontend_model)
        .map_err(|e| anyhow!("Invalid frontend agent model: {e}"))?;
    validate_model_name(&quality_model).map_err(|e| anyhow!("Invalid quality agent model: {e}"))?;
    validate_model_name(&security_model)
        .map_err(|e| anyhow!("Invalid security agent model: {e}"))?;
    validate_model_name(&testing_model).map_err(|e| anyhow!("Invalid testing agent model: {e}"))?;

    let implementation_max_retries =
        parse_max_retries_argument(arguments, "implementation_max_retries")
            .or(parse_max_retries_argument(arguments, "factory_max_retries"))
            .or(parse_max_retries_argument(
                arguments,
                "opencode_max_retries",
            ))
            .or(implementation_agent_max_retries)
            .or(effective_config.defaults.play.implementation_max_retries)
            .or(effective_config.defaults.play.max_retries)
            .or(effective_config.defaults.code.max_retries)
            .unwrap_or(10);

    let frontend_max_retries = parse_max_retries_argument(arguments, "frontend_max_retries")
        .or(frontend_agent_max_retries)
        .or(effective_config.defaults.play.frontend_max_retries)
        .or(effective_config.defaults.play.max_retries)
        .or(effective_config.defaults.code.max_retries)
        .unwrap_or(10);

    let quality_max_retries = parse_max_retries_argument(arguments, "quality_max_retries")
        .or(quality_agent_max_retries)
        .or(effective_config.defaults.play.quality_max_retries)
        .or(effective_config.defaults.play.max_retries)
        .or(effective_config.defaults.code.max_retries)
        .unwrap_or(10);

    let security_max_retries = parse_max_retries_argument(arguments, "security_max_retries")
        .or(security_agent_max_retries)
        .or(effective_config.defaults.play.security_max_retries)
        .or(effective_config.defaults.play.max_retries)
        .or(effective_config.defaults.code.max_retries)
        .unwrap_or(10);

    let testing_max_retries = parse_max_retries_argument(arguments, "testing_max_retries")
        .or(testing_agent_max_retries)
        .or(effective_config.defaults.play.testing_max_retries)
        .or(effective_config.defaults.play.max_retries)
        .or(effective_config.defaults.code.max_retries)
        .unwrap_or(10);

    let opencode_max_retries_override =
        parse_max_retries_argument(arguments, "opencode_max_retries");
    let opencode_max_retries = opencode_max_retries_override.unwrap_or(implementation_max_retries);

    // Check for requirements.yaml file
    // Try to determine workspace directory, but don't fail if we can't
    let workspace_dir_result =
        resolve_workspace_dir().ok_or_else(|| anyhow!("Workspace directory not found"));

    // Only check for requirements if we have a valid workspace directory
    let requirements_path = if let Ok(workspace_dir) = workspace_dir_result {
        let docs_dir = workspace_dir.join(docs_project_directory);
        let task_requirements_path = docs_dir.join(format!("task-{task_id}/requirements.yaml"));
        let project_requirements_path = docs_dir.join("requirements.yaml");

        eprintln!(
            "🔍 Checking for requirements.yaml in: {} (docs_project_directory='{}')",
            docs_dir.display(),
            docs_project_directory
        );

        if task_requirements_path.exists() {
            eprintln!("📋 Found task-specific requirements.yaml for task {task_id}");
            Some(task_requirements_path)
        } else if project_requirements_path.exists() {
            eprintln!("📋 Found project-level requirements.yaml");
            Some(project_requirements_path)
        } else {
            eprintln!("ℹ️ No requirements.yaml found");
            None
        }
    } else {
        eprintln!("⚠️ Could not determine workspace directory, skipping requirements check");
        None
    };

    let mut params = vec![
        format!("task-id={task_id}"),
        format!("repository={repository}"),
        format!("service={service}"),
        format!("docs-repository={docs_repository}"),
        format!("docs-project-directory={docs_project_directory}"),
        format!("implementation-agent={implementation_agent}"),
        format!("implementation-cli={implementation_cli}"),
        format!("implementation-model={implementation_model}"),
        format!("implementation-tools={implementation_tools}"),
        format!("implementation-model-rotation={implementation_model_rotation}"),
        format!("frontend-agent={frontend_agent}"),
        format!("frontend-cli={frontend_cli}"),
        format!("frontend-model={frontend_model}"),
        format!("frontend-tools={frontend_tools}"),
        format!("frontend-model-rotation={frontend_model_rotation}"),
        format!("quality-agent={quality_agent}"),
        format!("quality-cli={quality_cli}"),
        format!("quality-model={quality_model}"),
        format!("quality-tools={quality_tools}"),
        format!("quality-model-rotation={quality_model_rotation}"),
        format!("security-agent={security_agent}"),
        format!("security-cli={security_cli}"),
        format!("security-model={security_model}"),
        format!("security-tools={security_tools}"),
        format!("security-model-rotation={security_model_rotation}"),
        format!("testing-agent={testing_agent}"),
        format!("testing-cli={testing_cli}"),
        format!("testing-model={testing_model}"),
        format!("testing-tools={testing_tools}"),
        format!("testing-model-rotation={testing_model_rotation}"),
    ];

    params.push(format!(
        "implementation-max-retries={implementation_max_retries}"
    ));
    params.push(format!("frontend-max-retries={frontend_max_retries}"));
    params.push(format!("quality-max-retries={quality_max_retries}"));
    params.push(format!("security-max-retries={security_max_retries}"));
    params.push(format!("testing-max-retries={testing_max_retries}"));
    params.push(format!("opencode-max-retries={opencode_max_retries}"));

    // Auto-merge parameter
    let auto_merge = parse_bool_argument(arguments, "auto_merge")
        .or(effective_config.defaults.play.auto_merge)
        .unwrap_or(false);
    params.push(format!("auto-merge={auto_merge}"));

    // Parallel execution parameter - determines which workflow template to use
    let parallel_execution = parse_bool_argument(arguments, "parallel_execution")
        .or(effective_config.defaults.play.parallel_execution)
        .unwrap_or(false);

    // Select workflow template based on parallel_execution flag
    let workflow_template = if parallel_execution {
        eprintln!("🚀 Using parallel execution mode (play-project-workflow-template)");
        "workflowtemplate/play-project-workflow-template"
    } else {
        eprintln!("🔄 Using sequential execution mode (play-workflow-template)");
        "workflowtemplate/play-workflow-template"
    };

    // Add parallel-execution parameter for the workflow
    params.push(format!("parallel-execution={parallel_execution}"));

    // Final task parameter - indicates this is the last task requiring deployment verification
    let final_task = parse_bool_argument(arguments, "final_task").unwrap_or(false);
    params.push(format!("final-task={final_task}"));

    // Load and encode requirements.yaml if it exists
    if let Some(path) = requirements_path {
        let requirements_content = std::fs::read_to_string(&path).context(format!(
            "Failed to read requirements file: {}",
            path.display()
        ))?;

        // Base64 encode the requirements
        let encoded = general_purpose::STANDARD.encode(requirements_content);
        params.push(format!("task-requirements={encoded}"));
        eprintln!("✅ Encoded requirements.yaml for workflow");
    } else {
        // Always provide task-requirements parameter, even if empty (Argo requires it)
        params.push("task-requirements=".to_string());
    }

    // Build labels for workflow tracking
    let repo_label = format!("repository={}", repository.replace('/', "-"));
    let workflow_type_label = "workflow-type=play".to_string();
    let task_id_label = format!("task-id={task_id}");

    // CLEANUP: Delete old play workflows for this repository before starting new one
    // This ensures old GitHub checks get cancelled properly
    eprintln!("🧹 Checking for old play workflows to clean up...");
    let cleanup_result = run_argo_cli(&[
        "list",
        "-n",
        "cto",
        "-l",
        &repo_label,
        "-l",
        &workflow_type_label,
        "-o",
        "json",
    ]);

    if let Ok(workflows_json) = cleanup_result {
        if let Ok(workflows) = serde_json::from_str::<serde_json::Value>(&workflows_json) {
            if let Some(items) = workflows.get("items").and_then(|v| v.as_array()) {
                let now = std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap()
                    .as_secs();

                for workflow in items {
                    if let (Some(name), Some(created_at), phase) = (
                        workflow["metadata"]["name"].as_str(),
                        workflow["metadata"]["creationTimestamp"].as_str(),
                        workflow["status"]["phase"].as_str(),
                    ) {
                        // Parse RFC3339 timestamp
                        if let Ok(created_time) = chrono::DateTime::parse_from_rfc3339(created_at) {
                            let created_secs = created_time.timestamp();
                            // Only process workflows with valid (non-negative) timestamps
                            if created_secs >= 0 {
                                #[allow(clippy::cast_sign_loss)]
                                let created_secs_u64 = created_secs as u64;

                                // Handle clock skew: if workflow timestamp is in the future, treat as age 0
                                let age_secs = if created_secs_u64 > now {
                                    eprintln!(
                                        "  ⚠️  Workflow has future timestamp (clock skew detected): {name}"
                                    );
                                    0
                                } else {
                                    now - created_secs_u64
                                };

                                // Skip workflows created within the last 10 seconds to avoid race conditions
                                if age_secs < 10 {
                                    eprintln!(
                                        "  ⏭️  Skipping recent workflow ({age_secs}s old): {name}"
                                    );
                                    continue;
                                }

                                // Check workflow status - only delete completed/failed workflows
                                // Skip running, pending, or uninitialized workflows to avoid data loss
                                let phase_lower = phase.map(str::to_lowercase);
                                match phase_lower.as_deref() {
                                    Some("running" | "pending") => {
                                        eprintln!(
                                            "  ⏭️  Skipping active workflow (status: {phase:?}): {name}"
                                        );
                                    }
                                    Some("succeeded" | "failed" | "error") => {
                                        eprintln!(
                                            "  🗑️  Deleting completed workflow ({age_secs}s old, status: {phase:?}): {name}"
                                        );
                                        let _ = run_argo_cli(&["stop", name, "-n", "cto"]);
                                        let _ = run_argo_cli(&["delete", name, "-n", "cto"]);
                                    }
                                    None => {
                                        eprintln!(
                                            "  ⏭️  Skipping workflow with no phase (may be initializing): {name}"
                                        );
                                    }
                                    Some(other) => {
                                        eprintln!(
                                            "  ⏭️  Skipping workflow with unknown status '{other}': {name}"
                                        );
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    let mut args: Vec<&str> = vec!["submit", "--from", workflow_template, "-n", "cto"];

    // Add labels for workflow tracking (enables auto-detection)
    args.push("-l");
    args.push(&repo_label);
    args.push("-l");
    args.push(&workflow_type_label);
    args.push("-l");
    args.push(&task_id_label);

    // Add all parameters to the command
    for param in &params {
        args.push("-p");
        args.push(param);
    }

    match run_argo_cli(&args) {
        Ok(output) => {
            // Extract workflow name from output
            let workflow_name = if let Ok(wf_json) = serde_json::from_str::<Value>(&output) {
                wf_json
                    .get("metadata")
                    .and_then(|m| m.get("name"))
                    .and_then(|n| n.as_str())
                    .map(String::from)
            } else {
                None
            };

            // Write progress ConfigMap if we got a workflow name
            if let Some(ref wf_name) = workflow_name {
                let progress = PlayProgress {
                    repository: repository.clone(),
                    branch: "main".to_string(),
                    current_task_id: Some(task_id),
                    workflow_name: Some(wf_name.clone()),
                    status: PlayStatus::InProgress,
                    stage: Some("implementation".to_string()),
                };

                if let Err(e) = write_play_progress(&progress) {
                    eprintln!("⚠️  Failed to write progress ConfigMap: {e}");
                    eprintln!("   (This won't affect workflow execution)");
                }
            }

            Ok(json!({
                "success": true,
                "message": "Play workflow submitted successfully",
                "output": output,
                "task_id": task_id,
                "repository": repository,
                "service": service,
                "docs_repository": docs_repository,
                "docs_project_directory": docs_project_directory,
                "implementation_agent": implementation_agent,
                "implementation_cli": implementation_cli,
                "implementation_model": implementation_model,
                "quality_agent": quality_agent,
                "quality_cli": quality_cli,
                "quality_model": quality_model,
                "security_agent": security_agent,
                "security_cli": security_cli,
                "security_model": security_model,
                "testing_agent": testing_agent,
                "testing_cli": testing_cli,
                "testing_model": testing_model,
                "model": implementation_model,
                "parameters": params,
                "workflow_name": workflow_name,
            }))
        }
        Err(e) => Err(anyhow!("Failed to submit play workflow: {e}")),
    }
}

/// Unified intake workflow - parses PRD, generates tasks, and creates documentation
/// This replaces the separate `intake_prd` and docs workflows
#[allow(clippy::disallowed_macros)]
#[allow(clippy::too_many_lines)]
fn handle_intake_workflow(arguments: &HashMap<String, Value>) -> Result<Value> {
    eprintln!("🚀 Processing unified intake request (PRD parsing + documentation generation)");

    // Get workspace directory from Cursor environment
    let workspace_dir = resolve_workspace_dir().unwrap_or_else(|| std::path::PathBuf::from("."));

    eprintln!("🔍 Using workspace directory: {}", workspace_dir.display());

    // Get project name (required)
    let project_name = arguments
        .get("project_name")
        .and_then(|v| v.as_str())
        .ok_or_else(|| anyhow!("project_name is required"))?;

    // Read PRD from project root or intake folder (root preferred), or use provided content
    let project_path = workspace_dir.join(project_name);
    let intake_path = project_path.join("intake");
    let prd_file_root = project_path.join("prd.txt");
    let prd_file_intake = intake_path.join("prd.txt");

    let prd_content = if let Some(content) = arguments.get("prd_content").and_then(|v| v.as_str()) {
        // Allow override via parameter for compatibility
        content.to_string()
    } else if prd_file_root.exists() {
        eprintln!("📋 Reading PRD from {project_name}/prd.txt");
        std::fs::read_to_string(&prd_file_root)
            .with_context(|| format!("Failed to read {project_name}/prd.txt"))?
    } else if prd_file_intake.exists() {
        eprintln!("📋 Reading PRD from {project_name}/intake/prd.txt");
        std::fs::read_to_string(&prd_file_intake)
            .with_context(|| format!("Failed to read {project_name}/intake/prd.txt"))?
    } else {
        return Err(anyhow!(
            "No PRD found. Please create either {project_name}/prd.txt or {project_name}/intake/prd.txt, or provide prd_content parameter"
        ));
    };

    // Read optional architecture file (prefer root, then intake)
    let arch_file_root = project_path.join("architecture.md");
    let arch_file_intake = intake_path.join("architecture.md");
    let architecture_content = if let Some(content) = arguments
        .get("architecture_content")
        .and_then(|v| v.as_str())
    {
        content.to_string()
    } else if arch_file_root.exists() {
        eprintln!("🏗️ Reading architecture from {project_name}/architecture.md");
        std::fs::read_to_string(&arch_file_root)
            .with_context(|| format!("Failed to read {project_name}/architecture.md"))?
    } else if arch_file_intake.exists() {
        eprintln!("🏗️ Reading architecture from {project_name}/intake/architecture.md");
        std::fs::read_to_string(&arch_file_intake)
            .with_context(|| format!("Failed to read {project_name}/intake/architecture.md"))?
    } else {
        String::new()
    };

    // Get configuration
    let config = CTO_CONFIG
        .get()
        .ok_or_else(|| anyhow!("Configuration not loaded"))?;

    // Auto-detect repository from git (using workspace directory)
    eprintln!("🔍 Auto-detecting repository from git...");
    let repository_name = get_git_repository_url_in_dir(Some(&workspace_dir))?;
    eprintln!("📦 Using repository: {repository_name}");
    let repository_url = format!("https://github.com/{repository_name}");

    // Auto-detect current branch (using workspace directory)
    eprintln!("🌿 Auto-detecting git branch...");
    let branch = get_git_current_branch_in_dir(Some(&workspace_dir))?;
    eprintln!("🎯 Using branch: {branch}");

    // Use configuration values with defaults (client can override)
    let github_app = arguments
        .get("github_app")
        .and_then(|v| v.as_str())
        .unwrap_or(&config.defaults.intake.github_app);

    // Extract model configuration (client can specify granular control)
    let primary_model = arguments
        .get("primary_model")
        .and_then(|v| v.as_str())
        .unwrap_or(&config.defaults.intake.primary.model);
    let research_model = arguments
        .get("research_model")
        .and_then(|v| v.as_str())
        .unwrap_or(&config.defaults.intake.research.model);
    let fallback_model = arguments
        .get("fallback_model")
        .and_then(|v| v.as_str())
        .unwrap_or(&config.defaults.intake.fallback.model);

    // Extract provider configuration
    let primary_provider = arguments
        .get("primary_provider")
        .and_then(|v| v.as_str())
        .unwrap_or(&config.defaults.intake.primary.provider);
    let research_provider = arguments
        .get("research_provider")
        .and_then(|v| v.as_str())
        .unwrap_or(&config.defaults.intake.research.provider);
    let fallback_provider = arguments
        .get("fallback_provider")
        .and_then(|v| v.as_str())
        .unwrap_or(&config.defaults.intake.fallback.provider);
    let num_tasks = 50; // Standard task count
    let expand_tasks = true; // Always expand for detailed planning
    let analyze_complexity = true; // Always analyze for better breakdown

    // Unified intake parameters (docs generation)
    let enrich_context = arguments
        .get("enrich_context")
        .and_then(Value::as_bool)
        .unwrap_or(true); // Default to true - auto-scrape URLs via Firecrawl
    let include_codebase = arguments
        .get("include_codebase")
        .and_then(Value::as_bool)
        .unwrap_or(config.defaults.docs.include_codebase);
    let docs_model = arguments
        .get("model")
        .and_then(|v| v.as_str())
        .unwrap_or(&config.defaults.docs.model);

    // CLI for documentation generation
    let cli = arguments
        .get("cli")
        .and_then(|v| v.as_str())
        .unwrap_or("claude");

    eprintln!("🤖 Using GitHub App: {github_app}");
    eprintln!("🧠 Using Primary Model: {primary_model} ({primary_provider})");
    eprintln!("🔬 Using Research Model: {research_model} ({research_provider})");
    eprintln!("🛡️  Using Fallback Model: {fallback_model} ({fallback_provider})");
    eprintln!("📚 Docs Model: {docs_model}");
    eprintln!("🔗 Context Enrichment (Firecrawl): {enrich_context}");
    eprintln!("📁 Include Codebase: {include_codebase}");
    eprintln!("🖥️  CLI for Docs: {cli}");

    // Create a ConfigMap with the intake files to avoid YAML escaping issues
    let configmap_name = format!(
        "intake-{}-{}",
        project_name.to_lowercase().replace(' ', "-"),
        chrono::Utc::now().timestamp()
    );

    eprintln!("📦 Creating ConfigMap: {configmap_name}");

    // Create ConfigMap with the intake content (unified: PRD + docs generation)
    let config_json = serde_json::json!({
        "project_name": project_name,
        "repository_url": format!("https://github.com/{}", repository_name),
        "github_app": github_app,
        "primary_model": primary_model,
        "research_model": research_model,
        "fallback_model": fallback_model,
        "primary_provider": primary_provider,
        "research_provider": research_provider,
        "fallback_provider": fallback_provider,
        "model": primary_model, // Legacy compatibility
        "num_tasks": num_tasks,
        "expand_tasks": expand_tasks,
        "analyze_complexity": analyze_complexity,
        // Unified intake: docs generation parameters
        "docs_model": docs_model,
        "enrich_context": enrich_context,
        "include_codebase": include_codebase,
        "cli": cli
    });

    // Create the ConfigMap using kubectl
    let kubectl_cmd = find_command("kubectl");
    let cm_output = std::process::Command::new(&kubectl_cmd)
        .args([
            "create",
            "configmap",
            &configmap_name,
            "-n",
            "cto",
            &format!("--from-literal=prd.txt={prd_content}"),
            &format!("--from-literal=architecture.md={architecture_content}"),
            &format!("--from-literal=config.json={config_json}"),
        ])
        .output();

    if let Err(e) = cm_output {
        return Err(anyhow!("Failed to create ConfigMap: {e}"));
    }

    if let Ok(output) = cm_output {
        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(anyhow!("Failed to create ConfigMap: {stderr}"));
        }
    }

    // Submit Argo workflow with minimal parameters
    let workflow_name = format!("intake-{}", chrono::Utc::now().timestamp());

    let argo_cmd = find_command("argo");
    let output = std::process::Command::new(&argo_cmd)
        .args([
            "submit",
            "--from",
            "workflowtemplate/project-intake",
            "-n",
            "cto",
            "--name",
            &workflow_name,
            "-p",
            &format!("configmap-name={configmap_name}"),
            "-p",
            &format!("project-name={project_name}"),
            "-p",
            &format!("repository-url={repository_url}"),
            "-p",
            &format!("source-branch={branch}"),
            "-p",
            &format!("github-app={github_app}"),
            "-p",
            &format!("primary-model={primary_model}"),
            "-p",
            &format!("research-model={research_model}"),
            "-p",
            &format!("fallback-model={fallback_model}"),
            "-p",
            &format!("primary-provider={primary_provider}"),
            "-p",
            &format!("research-provider={research_provider}"),
            "-p",
            &format!("fallback-provider={fallback_provider}"),
            "-p",
            &format!("num-tasks={num_tasks}"),
            "-p",
            &format!("expand-tasks={expand_tasks}"),
            "-p",
            &format!("analyze-complexity={analyze_complexity}"),
            // Unified intake: docs generation parameters
            "-p",
            &format!("docs-model={docs_model}"),
            "-p",
            &format!("enrich-context={enrich_context}"),
            "-p",
            &format!("include-codebase={include_codebase}"),
            "-p",
            &format!("cli={cli}"),
            "--wait=false",
            "-o",
            "json",
        ])
        .output();

    // Determine source labels for reporting
    let prd_source_label = if arguments
        .get("prd_content")
        .and_then(|v| v.as_str())
        .is_some()
    {
        "provided"
    } else if prd_file_root.exists() {
        "prd.txt"
    } else if prd_file_intake.exists() {
        "intake/prd.txt"
    } else {
        // Should be unreachable due to earlier validation
        "provided"
    };

    let architecture_source_label = if arguments
        .get("architecture_content")
        .and_then(|v| v.as_str())
        .is_some()
    {
        "provided"
    } else if arch_file_root.exists() {
        "architecture.md"
    } else if arch_file_intake.exists() {
        "intake/architecture.md"
    } else {
        "none"
    };

    match output {
        Ok(result) if result.status.success() => {
            let workflow_json: Value = serde_json::from_slice(&result.stdout)
                .unwrap_or_else(|_| json!({"message": "Workflow submitted"}));

            eprintln!("✅ Project intake workflow submitted: {workflow_name}");

            Ok(json!({
                "status": "submitted",
                "workflow_name": workflow_name,
                "workflow": workflow_json,
                "message": format!(
                    "Project intake initiated for '{}'. PR will be created in {} on branch '{}'",
                    project_name, repository_name, branch
                ),
                "details": {
                    "project_name": project_name,
                    "repository": repository_name,
                    "branch": branch,
                    "prd_source": prd_source_label,
                    "architecture_source": architecture_source_label
                }
            }))
        }
        Ok(result) => {
            let error_msg = String::from_utf8_lossy(&result.stderr);
            eprintln!("❌ Failed to submit intake workflow: {error_msg}");
            Err(anyhow!("Failed to submit intake workflow: {error_msg}"))
        }
        Err(e) => {
            eprintln!("❌ Failed to execute argo command: {e}");
            Err(anyhow!("Failed to execute argo command: {e}"))
        }
    }
}
fn handle_tool_calls(method: &str, params_map: &HashMap<String, Value>) -> Option<Result<Value>> {
    match method {
        "tools/call" => {
            let name = params_map
                .get("name")
                .and_then(|v| v.as_str())
                .ok_or(anyhow!("Missing tool name"));

            let arguments = params_map
                .get("arguments")
                .and_then(|v| v.as_object())
                .map(|obj| obj.iter().map(|(k, v)| (k.clone(), v.clone())).collect())
                .unwrap_or_default();

            match name {
                // Unified intake tool - combines PRD parsing and docs generation
                Ok("intake") => Some(handle_intake_workflow(&arguments).map(|result| json!({
                    "content": [{
                        "type": "text",
                        "text": serde_json::to_string_pretty(&result).unwrap_or_else(|_| result.to_string())
                    }]
                }))),
                Ok("play") => Some(handle_play_workflow(&arguments).map(|result| json!({
                    "content": [{
                        "type": "text",
                        "text": serde_json::to_string_pretty(&result).unwrap_or_else(|_| result.to_string())
                    }]
                }))),
                Ok("play_status") => Some(handle_play_status(&arguments).map(|result| json!({
                    "content": [{
                        "type": "text",
                        "text": serde_json::to_string_pretty(&result).unwrap_or_else(|_| result.to_string())
                    }]
                }))),
                Ok("jobs") => {
                    let result = handle_jobs_tool(&arguments);
                    Some(Ok(json!({
                        "content": [{
                            "type": "text",
                            "text": serde_json::to_string_pretty(&result).unwrap_or_else(|_| result.to_string())
                        }]
                    })))
                },
                Ok("stop_job") => Some(handle_stop_job_tool(&arguments).map(|result| json!({
                    "content": [{
                        "type": "text",
                        "text": serde_json::to_string_pretty(&result).unwrap_or_else(|_| result.to_string())
                    }]
                }))),
                Ok("input") => Some(handle_send_job_input(&arguments).map(|result| json!({
                    "content": [{
                        "type": "text",
                        "text": serde_json::to_string_pretty(&result).unwrap_or_else(|_| result.to_string())
                    }]
                }))),
                Ok("docs_ingest") => Some(handle_docs_ingest_tool(&arguments).map(|result| json!({
                    "content": [{
                        "type": "text",
                        "text": serde_json::to_string_pretty(&result).unwrap_or_else(|_| result.to_string())
                    }]
                }))),
                Ok("add_mcp_server") => Some(handle_add_mcp_server(&arguments).map(|result| json!({
                    "content": [{
                        "type": "text",
                        "text": serde_json::to_string_pretty(&result).unwrap_or_else(|_| result.to_string())
                    }]
                }))),
                Ok("remove_mcp_server") => Some(handle_remove_mcp_server(&arguments).map(|result| json!({
                    "content": [{
                        "type": "text",
                        "text": serde_json::to_string_pretty(&result).unwrap_or_else(|_| result.to_string())
                    }]
                }))),
                Ok("update_mcp_server") => Some(handle_update_mcp_server(&arguments).map(|result| json!({
                    "content": [{
                        "type": "text",
                        "text": serde_json::to_string_pretty(&result).unwrap_or_else(|_| result.to_string())
                    }]
                }))),
                Ok(unknown) => Some(Err(anyhow!("Unknown tool: {unknown}"))),
                Err(e) => Some(Err(e)),
            }
        }
        _ => None,
    }
}

fn handle_method(method: &str, params: Option<&Value>) -> Option<Result<Value>> {
    let params_map = extract_params(params);

    // Try MCP protocol methods first
    if let Some(result) = handle_mcp_methods(method, &params_map) {
        return Some(result);
    }

    // Handle notifications (no response)
    if method.starts_with("notifications/") {
        return None;
    }

    // Try tool calls
    if let Some(result) = handle_tool_calls(method, &params_map) {
        return Some(result);
    }

    Some(Err(anyhow!("Unknown method: {method}")))
}

fn run_kubectl_json(args: &[&str]) -> Result<Value> {
    let kubectl_cmd = find_command("kubectl");
    let output = std::process::Command::new(&kubectl_cmd)
        .args(args)
        .output()?;
    if output.status.success() {
        let stdout = String::from_utf8(output.stdout)?;
        let v: Value = serde_json::from_str(&stdout)?;
        Ok(v)
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr);
        Err(anyhow!("kubectl failed: {stderr}"))
    }
}

fn handle_jobs_tool(arguments: &std::collections::HashMap<String, Value>) -> Value {
    let namespace = arguments
        .get("namespace")
        .and_then(|v| v.as_str())
        .unwrap_or("cto");

    let include = arguments.get("include").and_then(|v| v.as_array());
    let include_play =
        include.is_none() || include.unwrap().iter().any(|x| x.as_str() == Some("play"));
    let include_intake = include.is_none()
        || include
            .unwrap()
            .iter()
            .any(|x| x.as_str() == Some("intake"));

    let mut jobs: Vec<Value> = Vec::new();

    // List all Argo workflows
    if let Ok(list_str) = run_argo_cli(&["list", "-n", namespace, "-o", "json"]) {
        if let Ok(v) = serde_json::from_str::<Value>(&list_str) {
            if let Some(items) = v.get("items").and_then(|v| v.as_array()) {
                for item in items {
                    let name = item
                        .get("metadata")
                        .and_then(|m| m.get("name"))
                        .and_then(|n| n.as_str())
                        .unwrap_or("");
                    let phase = item
                        .get("status")
                        .and_then(|s| s.get("phase"))
                        .and_then(|p| p.as_str())
                        .unwrap_or("");

                    // Determine workflow type based on name pattern
                    let workflow_type = if name.contains("play-workflow") {
                        "play"
                    } else if name.contains("intake") {
                        "intake"
                    } else {
                        "workflow"
                    };

                    // Only include if the type is requested
                    let should_include = match workflow_type {
                        "play" => include_play,
                        "intake" => include_intake,
                        _ => true, // Include other workflows by default
                    };

                    if should_include {
                        jobs.push(json!({
                            "type": workflow_type,
                            "name": name,
                            "namespace": namespace,
                            "phase": phase,
                            "status": item.get("status")
                        }));
                    }
                }
            }
        }
    }

    json!({
        "success": true,
        "namespace": namespace,
        "count": jobs.len(),
        "jobs": jobs
    })
}

fn handle_stop_job_tool(arguments: &std::collections::HashMap<String, Value>) -> Result<Value> {
    let job_type = arguments
        .get("job_type")
        .and_then(|v| v.as_str())
        .ok_or(anyhow!("job_type is required"))?;
    let name = arguments
        .get("name")
        .and_then(|v| v.as_str())
        .ok_or(anyhow!("name is required"))?;
    let namespace = arguments
        .get("namespace")
        .and_then(|v| v.as_str())
        .unwrap_or("cto");

    match job_type {
        "intake" => {
            // Terminate and delete intake workflow
            let _ = run_argo_cli(&["terminate", name, "-n", namespace]);
            match run_argo_cli(&["delete", name, "-n", namespace]) {
                Ok(msg) => Ok(
                    json!({"success": true, "message": format!("Deleted intake workflow {name}: {msg}"), "namespace": namespace}),
                ),
                Err(e) => Err(anyhow!(format!(
                    "Failed to delete intake workflow {name}: {e}"
                ))),
            }
        }
        "play" => {
            // Stop play workflow using Argo CLI
            match run_argo_cli(&["stop", name, "-n", namespace]) {
                Ok(_msg) => Ok(
                    json!({"success": true, "message": format!("Stopped play workflow {name}"), "namespace": namespace}),
                ),
                Err(e) => Err(anyhow!(format!("Failed to stop play workflow {name}: {e}"))),
            }
        }
        "workflow" => {
            // Stop generic workflow using Argo CLI
            match run_argo_cli(&["stop", name, "-n", namespace]) {
                Ok(_msg) => Ok(
                    json!({"success": true, "message": format!("Stopped workflow {name}"), "namespace": namespace}),
                ),
                Err(e) => Err(anyhow!(format!("Failed to stop workflow {name}: {e}"))),
            }
        }
        other => Err(anyhow!(format!(
            "Unsupported job_type: {other}. Supported types: intake, play, workflow"
        ))),
    }
}

#[allow(dead_code)]
fn handle_anthropic_message_tool(
    arguments: &std::collections::HashMap<String, Value>,
) -> Result<Value> {
    let api_key = std::env::var("ANTHROPIC_API_KEY")
        .map_err(|_| anyhow!("ANTHROPIC_API_KEY environment variable not set"))?;
    let model = arguments
        .get("model")
        .and_then(|v| v.as_str())
        .ok_or(anyhow!("model is required"))?;
    let system = arguments.get("system").and_then(|v| v.as_str());
    #[allow(clippy::cast_possible_truncation)]
    let max_tokens = arguments
        .get("max_tokens")
        .and_then(serde_json::Value::as_u64)
        .unwrap_or(1024) as u32;

    let messages = if let Some(raw) = arguments.get("messages").and_then(|v| v.as_array()) {
        raw.clone()
    } else {
        let mut content_parts: Vec<Value> = Vec::new();
        if let Some(input_json) = arguments.get("input_json") {
            content_parts.push(json!({"type": "input_json", "input_json": input_json}));
        }
        if let Some(text) = arguments.get("text").and_then(|v| v.as_str()) {
            content_parts.push(json!({"type": "text", "text": text}));
        }
        if content_parts.is_empty() {
            return Err(anyhow!("Provide either messages, text, or input_json"));
        }
        vec![json!({"role": "user", "content": Value::Array(content_parts)})]
    };

    // Use blocking reqwest to avoid async changes in this server
    let client = reqwest::blocking::Client::new();
    let mut body = json!({
        "model": model,
        "messages": messages,
        "max_tokens": max_tokens
    });
    if let Some(s) = system {
        body["system"] = json!(s);
    }

    let resp = client
        .post("https://api.anthropic.com/v1/messages")
        .header("x-api-key", api_key)
        .header("anthropic-version", "2023-06-01")
        .json(&body)
        .send()
        .and_then(reqwest::blocking::Response::error_for_status)
        .map_err(|e| anyhow!(format!("Anthropic request failed: {e}")))?;

    let json_resp: Value = resp
        .json()
        .map_err(|e| anyhow!(format!("Failed to parse Anthropic response: {e}")))?;
    Ok(json_resp)
}

fn handle_send_job_input(arguments: &std::collections::HashMap<String, Value>) -> Result<Value> {
    // Inputs
    let namespace = arguments
        .get("namespace")
        .and_then(|v| v.as_str())
        .unwrap_or("cto");
    let text = arguments
        .get("text")
        .and_then(|v| v.as_str())
        .ok_or(anyhow!("text is required"))?;
    let job_type = arguments
        .get("job_type")
        .and_then(|v| v.as_str())
        .unwrap_or("code");
    let user = arguments.get("user").and_then(|v| v.as_str());
    let service = arguments.get("service").and_then(|v| v.as_str());

    // Build service selector based on available parameters
    let mut label_selectors = vec![
        format!("agents.platform/input=bridge"),
        format!("agents.platform/jobType={}", job_type),
    ];

    // Add user filter if provided
    if let Some(user_label) = user {
        label_selectors.push(format!("agents.platform/user={user_label}"));
    }

    // Add service/name filter if provided
    if let Some(service_name) = service {
        label_selectors.push(format!("agents.platform/name={service_name}"));
    }

    let selector = label_selectors.join(",");

    // Find services (not pods) by labels
    let services_json = run_kubectl_json(&[
        "get", "services", "-n", namespace, "-l", &selector, "-o", "json",
    ])?
    .to_string();
    let services: Value = serde_json::from_str(&services_json)?;
    let service_items = services
        .get("items")
        .and_then(|v| v.as_array())
        .ok_or(anyhow!("No services found"))?;

    if service_items.is_empty() {
        return Err(anyhow!(
            "No input bridge services found with selector: {selector}. Available parameters: job_type={job_type}, user={user:?}, service={service:?}"
        ));
    }

    // Take the first matching service (or could implement more sophisticated selection)
    let service_item = &service_items[0];
    let service_name = service_item
        .get("metadata")
        .and_then(|m| m.get("name"))
        .and_then(|n| n.as_str())
        .ok_or(anyhow!("Service missing name"))?;

    let service_port = service_item
        .get("spec")
        .and_then(|s| s.get("ports"))
        .and_then(|p| p.as_array())
        .and_then(|ports| ports.first())
        .and_then(|port| port.get("port"))
        .and_then(serde_json::Value::as_u64)
        .ok_or(anyhow!("Service missing port"))?;

    // Construct the service URL
    let service_url =
        format!("http://{service_name}.{namespace}.svc.cluster.local:{service_port}/input");

    // Format the message as JSON
    let message_data = json!({
        "text": text
    });

    // Send HTTP POST request to the input bridge service
    let client = reqwest::blocking::Client::new();
    let response = client
        .post(&service_url)
        .header("Content-Type", "application/json")
        .json(&message_data)
        .timeout(Duration::from_secs(30))
        .send()
        .context("Failed to send HTTP request to input bridge")?;

    let status = response.status();

    if status.is_success() {
        let response_text = response.text().unwrap_or_else(|_| "OK".to_string());
        Ok(json!({
            "success": true,
            "service": service_name,
            "url": service_url,
            "response": response_text
        }))
    } else {
        let error_text = response
            .text()
            .unwrap_or_else(|_| "Unknown error".to_string());
        Err(anyhow!(
            "HTTP request failed with status {status}: {error_text}"
        ))
    }
}

fn handle_docs_ingest_tool(arguments: &std::collections::HashMap<String, Value>) -> Result<Value> {
    let url = arguments
        .get("url")
        .and_then(|v| v.as_str())
        .ok_or(anyhow!("url is required"))?;

    let doc_type_str = arguments
        .get("type")
        .and_then(|v| v.as_str())
        .ok_or(anyhow!("type is required (repo or scrape)"))?;

    let doc_type = doc_proxy::DocType::from_str(doc_type_str)?;

    let query = arguments.get("query").and_then(|v| v.as_str());

    let limit = arguments
        .get("limit")
        .and_then(Value::as_u64)
        .and_then(|v| u32::try_from(v).ok())
        .unwrap_or(50);

    doc_proxy::handle_add_docs(url, doc_type, query, limit)
}

/// Create a `CodeRun` for MCP server management tasks
fn create_mcp_server_coderun(
    task_type: &str,
    server_key: &str,
    github_url: Option<&str>,
    readme_content: Option<&str>,
    skip_merge: bool,
) -> Result<String> {
    let kubectl_cmd = find_command("kubectl");

    // Build environment variables for the CodeRun
    let mut env_map = serde_json::Map::new();
    env_map.insert("MCP_SERVER_TASK".to_string(), json!(task_type));
    env_map.insert("MCP_SERVER_KEY".to_string(), json!(server_key));
    env_map.insert("SKIP_MERGE".to_string(), json!(skip_merge.to_string()));

    if let Some(url) = github_url {
        env_map.insert("MCP_SERVER_GITHUB_URL".to_string(), json!(url));
    }

    if let Some(readme) = readme_content {
        // Base64 encode the README content to avoid YAML escaping issues
        let encoded = base64::Engine::encode(&base64::engine::general_purpose::STANDARD, readme);
        env_map.insert("MCP_SERVER_README_B64".to_string(), json!(encoded));
    }

    // Build the CodeRun manifest
    let coderun = json!({
        "apiVersion": "agents.platform/v1",
        "kind": "CodeRun",
        "metadata": {
            "generateName": format!("coderun-mcp-{}-", task_type),
            "namespace": "cto",
            "labels": {
                "agent": "rex",
                "task-type": format!("mcp-server-{}", task_type),
                "mcp-server-key": server_key
            }
        },
        "spec": {
            "taskId": 0,
            "service": "cto",
            "repositoryUrl": "https://github.com/5dlabs/cto",
            "docsRepositoryUrl": "https://github.com/5dlabs/cto",
            "model": "claude-sonnet-4-5-20250929",
            "githubApp": "5DLabs-Rex",
            "continueSession": false,
            "overwriteMemory": true,
            "env": env_map
        }
    });

    // Write to temp file and apply
    let coderun_json = serde_json::to_string_pretty(&coderun)?;
    let mut temp_file = NamedTempFile::new()?;
    temp_file.write_all(coderun_json.as_bytes())?;
    temp_file.flush()?;

    let output = Command::new(&kubectl_cmd)
        .args([
            "create",
            "-f",
            temp_file.path().to_str().unwrap(),
            "-o",
            "jsonpath={.metadata.name}",
        ])
        .output()
        .map_err(|e| anyhow!("Failed to execute kubectl: {e}"))?;

    if output.status.success() {
        let name = String::from_utf8(output.stdout)?.trim().to_string();
        Ok(name)
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr);
        Err(anyhow!("Failed to create CodeRun: {stderr}"))
    }
}

fn handle_add_mcp_server(arguments: &std::collections::HashMap<String, Value>) -> Result<Value> {
    let github_url = arguments
        .get("github_url")
        .and_then(|v| v.as_str())
        .ok_or(anyhow!("github_url is required"))?;

    let skip_merge = arguments
        .get("skip_merge")
        .and_then(serde_json::Value::as_bool)
        .unwrap_or(false);

    // Parse and validate GitHub URL
    let (_org, repo) = doc_proxy::parse_github_url(github_url)?;
    let server_key = doc_proxy::derive_server_key(&repo);

    eprintln!("📦 Adding MCP server from {github_url}");
    eprintln!("   Derived server key: {server_key}");

    // Fetch README via Firecrawl
    eprintln!("📄 Fetching README from GitHub...");
    let readme_content = match doc_proxy::scrape_readme(github_url) {
        Ok(content) => {
            eprintln!("   ✅ README fetched ({} chars)", content.len());
            Some(content)
        }
        Err(e) => {
            eprintln!("   ⚠️ Could not fetch README: {e}");
            eprintln!("   Rex will attempt to fetch it directly");
            None
        }
    };

    // Create CodeRun for Rex
    eprintln!("🚀 Creating CodeRun for Rex...");
    let coderun_name = create_mcp_server_coderun(
        "add",
        &server_key,
        Some(github_url),
        readme_content.as_deref(),
        skip_merge,
    )?;

    eprintln!("   ✅ CodeRun created: {coderun_name}");

    Ok(json!({
        "success": true,
        "message": "CodeRun created - Rex will analyze README, update values.yaml, and create PR",
        "coderun_name": coderun_name,
        "github_url": github_url,
        "server_key": server_key,
        "skip_merge": skip_merge,
        "note": "Monitor the CodeRun for progress. After PR merge, a verification CodeRun will automatically confirm the server is available."
    }))
}

fn handle_remove_mcp_server(arguments: &std::collections::HashMap<String, Value>) -> Result<Value> {
    let server_key = arguments
        .get("server_key")
        .and_then(|v| v.as_str())
        .ok_or(anyhow!("server_key is required"))?;

    let skip_merge = arguments
        .get("skip_merge")
        .and_then(serde_json::Value::as_bool)
        .unwrap_or(false);

    eprintln!("🗑️ Removing MCP server: {server_key}");

    // Create CodeRun for Rex
    eprintln!("🚀 Creating CodeRun for Rex...");
    let coderun_name = create_mcp_server_coderun("remove", server_key, None, None, skip_merge)?;

    eprintln!("   ✅ CodeRun created: {coderun_name}");

    Ok(json!({
        "success": true,
        "message": "CodeRun created - Rex will remove server from values.yaml and create PR",
        "coderun_name": coderun_name,
        "server_key": server_key,
        "skip_merge": skip_merge,
        "note": "Monitor the CodeRun for progress."
    }))
}

fn handle_update_mcp_server(arguments: &std::collections::HashMap<String, Value>) -> Result<Value> {
    let server_key = arguments
        .get("server_key")
        .and_then(|v| v.as_str())
        .ok_or(anyhow!("server_key is required"))?;

    let github_url = arguments.get("github_url").and_then(|v| v.as_str());

    let skip_merge = arguments
        .get("skip_merge")
        .and_then(serde_json::Value::as_bool)
        .unwrap_or(false);

    eprintln!("🔄 Updating MCP server: {server_key}");

    // Fetch README if GitHub URL provided
    let readme_content = if let Some(url) = github_url {
        eprintln!("📄 Fetching README from {url}...");
        match doc_proxy::scrape_readme(url) {
            Ok(content) => {
                eprintln!("   ✅ README fetched ({} chars)", content.len());
                Some(content)
            }
            Err(e) => {
                eprintln!("   ⚠️ Could not fetch README: {e}");
                None
            }
        }
    } else {
        eprintln!("   ℹ️ No GitHub URL provided - Rex will use existing config");
        None
    };

    // Create CodeRun for Rex
    eprintln!("🚀 Creating CodeRun for Rex...");
    let coderun_name = create_mcp_server_coderun(
        "update",
        server_key,
        github_url,
        readme_content.as_deref(),
        skip_merge,
    )?;

    eprintln!("   ✅ CodeRun created: {coderun_name}");

    Ok(json!({
        "success": true,
        "message": "CodeRun created - Rex will update server configuration and create PR if changes needed",
        "coderun_name": coderun_name,
        "server_key": server_key,
        "github_url": github_url,
        "skip_merge": skip_merge,
        "note": "Monitor the CodeRun for progress."
    }))
}

#[allow(clippy::disallowed_macros)]
async fn rpc_loop() -> Result<()> {
    eprintln!("Starting RPC loop");
    let stdin = tokio::io::stdin();
    let reader = BufReader::new(stdin);
    let mut lines = reader.lines();
    let mut stdout = tokio::io::stdout();

    loop {
        // Add 30 second timeout for reading from stdin
        let line_result = timeout(Duration::from_secs(30), lines.next_line()).await;

        let line = match line_result {
            Ok(Ok(Some(line))) => line,
            Ok(Ok(None)) => {
                eprintln!("Stdin closed, exiting RPC loop");
                break;
            }
            Ok(Err(e)) => {
                eprintln!("Error reading from stdin: {e}");
                break;
            }
            Err(_) => {
                eprintln!("Timeout waiting for stdin, checking if we should exit...");
                // Check if stdin is still valid, if not exit gracefully
                continue;
            }
        };

        eprintln!("Received line: {line}");
        let request: RpcRequest = match serde_json::from_str(&line) {
            Ok(req) => req,
            Err(e) => {
                eprintln!("Invalid JSON request: {e}");
                continue;
            }
        };
        eprintln!("Parsed request for method: {}", request.method);

        let result = handle_method(&request.method, request.params.as_ref());
        if let Some(method_result) = result {
            let resp_json = match method_result {
                Ok(res) => {
                    let response = RpcSuccessResponse {
                        jsonrpc: "2.0".to_string(),
                        result: res,
                        id: request.id,
                    };
                    serde_json::to_string(&response)?
                }
                Err(err) => {
                    let response = RpcErrorResponse {
                        jsonrpc: "2.0".to_string(),
                        error: RpcError {
                            code: -32600,
                            message: err.to_string(),
                            data: None,
                        },
                        id: request.id,
                    };
                    serde_json::to_string(&response)?
                }
            };
            // Add timeout for stdout operations to prevent hanging
            if timeout(
                Duration::from_secs(5),
                stdout.write_all((resp_json + "\n").as_bytes()),
            )
            .await
            .is_err()
            {
                eprintln!("Timeout writing to stdout, exiting");
                break;
            }
            if timeout(Duration::from_secs(5), stdout.flush())
                .await
                .is_err()
            {
                eprintln!("Timeout flushing stdout, exiting");
                break;
            }
        }
    }
    Ok(())
}

#[allow(clippy::disallowed_macros)]
fn main() -> Result<()> {
    eprintln!(
        "🚀 Starting 5D Labs MCP Server... (built: {})",
        env!("BUILD_TIMESTAMP")
    );

    // Initialize configuration from JSON file
    let config = load_cto_config().context("Failed to load cto-config.json")?;
    eprintln!(
        "📋 Loaded {} agents from config: {:?}",
        config.agents.len(),
        config.agents.keys().collect::<Vec<_>>()
    );

    // Store in global static
    CTO_CONFIG
        .set(config)
        .map_err(|_| anyhow!("Failed to set CTO config"))?;
    eprintln!("✅ Configuration loaded");

    eprintln!("Creating runtime...");
    let rt = Runtime::new()?;
    eprintln!("Runtime created, starting RPC loop");

    // Set up signal handling for graceful shutdown
    rt.block_on(async {
        tokio::select! {
            result = rpc_loop() => {
                eprintln!("RPC loop completed with result: {result:?}");
                result
            }
            _ = signal::ctrl_c() => {
                eprintln!("Received Ctrl+C, shutting down gracefully");
                Ok(())
            }
        }
    })?;

    eprintln!("MCP server shutdown complete");
    Ok(())
}
