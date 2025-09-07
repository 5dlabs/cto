use anyhow::{anyhow, Context, Result};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::collections::HashMap;
use std::io::Write;
use std::path::Path;
use std::process::Command;
use std::sync::OnceLock;
use tempfile::NamedTempFile;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::runtime::Runtime;
use tokio::signal;
use tokio::time::{timeout, Duration};

mod tools;

// Global configuration loaded once at startup
static CTO_CONFIG: OnceLock<CtoConfig> = OnceLock::new();

#[derive(Debug, Deserialize, Clone)]
struct AgentConfig {
    #[serde(rename = "githubApp")]
    github_app: String,
    cli: String,
    model: String,
    #[allow(dead_code)]
    tools: AgentTools,
}

#[derive(Debug, Deserialize, Clone)]
struct AgentTools {
    #[allow(dead_code)]
    remote: Vec<String>,
    #[serde(rename = "localServers")]
    #[allow(dead_code)]
    local_servers: LocalServerConfig,
}

#[derive(Debug, Deserialize, Clone)]
struct LocalServerConfig {
    #[allow(dead_code)]
    filesystem: ServerConfig,
    #[allow(dead_code)]
    git: ServerConfig,
}

#[derive(Debug, Deserialize, Clone)]
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
    #[serde(default)]
    docs_ingest: DocsIngestDefaults,
}

#[derive(Debug, Deserialize, Clone)]
struct DocsDefaults {
    model: String,
    #[serde(rename = "githubApp")]
    github_app: String,
    #[serde(rename = "includeCodebase")]
    include_codebase: bool,
    #[serde(rename = "sourceBranch")]
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

/// Validate model name format (support both Claude API and CLAUDE code formats)
fn validate_model_name(model: &str) -> Result<()> {
    if !model.starts_with("claude-") && !["opus", "sonnet", "haiku"].contains(&model) {
        return Err(anyhow!(
            "Invalid model '{}'. Must be a valid Claude model name (claude-* format) or CLAUDE code model (opus, sonnet, haiku)",
            model
        ));
    }
    Ok(())
}

#[derive(Debug, Deserialize, Clone, Default)]
struct PlayDefaults {
    model: String,
    cli: String,
    #[serde(rename = "implementationAgent")]
    implementation_agent: String,
    #[serde(rename = "qualityAgent")]
    quality_agent: String,
    #[serde(rename = "testingAgent")]
    testing_agent: String,
    repository: Option<String>,
    service: Option<String>,
    #[serde(rename = "docsRepository")]
    docs_repository: Option<String>,
    #[serde(rename = "docsProjectDirectory")]
    docs_project_directory: Option<String>,
}

#[derive(Debug, Deserialize, Clone)]
struct DocsIngestDefaults {
    model: String,
    #[serde(rename = "docServerUrl")]
    doc_server_url: String,
}

impl Default for DocsIngestDefaults {
    fn default() -> Self {
        DocsIngestDefaults {
            model: "claude-sonnet-4-20250514".to_string(),
            // Use the internal Kubernetes service URL - accessible via Twingate
            doc_server_url: "http://doc-server-agent-docs-server.mcp.svc.cluster.local:80".to_string(),
        }
    }
}



/// Load configuration from cto-config.json file
/// Looks in current directory, workspace root, or WORKSPACE_FOLDER_PATHS for cto-config.json
#[allow(clippy::disallowed_macros)]
fn load_cto_config() -> Result<CtoConfig> {
    let mut config_paths = vec![
        std::path::PathBuf::from("cto-config.json"),
        std::path::PathBuf::from("../cto-config.json"),
    ];

    // TEMPORARY DEBUG: Print all environment variables
    eprintln!("🐛 DEBUG: Environment variables:");
    for (key, value) in std::env::vars() {
        eprintln!("🐛   {key}: {value}");
    }
    eprintln!(
        "🐛 DEBUG: Current working directory: {:?}",
        std::env::current_dir().unwrap_or_else(|e| {
            eprintln!("⚠️ Failed to get current directory: {e}");
            std::path::PathBuf::from(".")
        })
    );

    // Add workspace folder paths if available (Cursor provides this)
    if let Ok(workspace_paths) = std::env::var("WORKSPACE_FOLDER_PATHS") {
        eprintln!("🐛 DEBUG: WORKSPACE_FOLDER_PATHS found: {workspace_paths}");
        for workspace_path in workspace_paths.split(',') {
            let workspace_path = workspace_path.trim();
            eprintln!("🐛 DEBUG: Adding config path: {workspace_path}");
            config_paths.push(std::path::PathBuf::from(workspace_path).join("cto-config.json"));
        }
    } else {
        eprintln!("🐛 DEBUG: WORKSPACE_FOLDER_PATHS not found in environment");
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

    Err(anyhow!("cto-config.json not found in current directory or parent directory.{} Please create a configuration file in your project root.", workspace_info))
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
                "name": "agent-platform-mcp",
                "title": "Agent Platform MCP Server",
                "version": "1.0.0"
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

fn run_argo_cli(args: &[&str]) -> Result<String> {
    let output = Command::new("argo")
        .args(args)
        .output()
        .context("Failed to execute argo command")?;

    if output.status.success() {
        Ok(String::from_utf8(output.stdout)?.trim().to_string())
    } else {
        let stderr = String::from_utf8(output.stderr)?;
        Err(anyhow!("Argo command failed: {}", stderr))
    }
}

/// Get the remote URL for the current git repository
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
        Err(anyhow!("Git command failed: {}", stderr))
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
        Err(anyhow!("Git command failed: {}", stderr))
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
        return Err(anyhow!("Failed to get git repository URL: {}", stderr));
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

    Err(anyhow!("Could not parse repository URL: {}", url))
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

#[allow(clippy::disallowed_macros)]
fn handle_docs_workflow(arguments: &HashMap<String, Value>) -> Result<Value> {
    let working_directory = arguments
        .get("working_directory")
        .and_then(|v| v.as_str())
        .ok_or(anyhow!("Missing required parameter: working_directory"))?;

    let config = CTO_CONFIG.get().unwrap();

    // Get workspace directory from Cursor environment, then navigate to working_directory
    let workspace_dir = std::env::var("WORKSPACE_FOLDER_PATHS")
        .map(|paths| {
            let first_path = paths.split(',').next().unwrap_or(&paths).trim();
            first_path.to_string()
        })
        .map(std::path::PathBuf::from)
        .unwrap_or_else(|_| std::env::current_dir().unwrap_or_default());

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
        git_root = project_dir.clone();
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
        .map(String::from)
        .unwrap_or_else(|| config.defaults.docs.source_branch.clone());

    // Check for uncommitted changes and push them before starting docs generation
    eprintln!("🔍 Checking for uncommitted changes...");
    eprintln!(
        "🐛 DEBUG: Current directory for git: {:?}",
        std::env::current_dir().unwrap_or_else(|_| std::path::PathBuf::from("."))
    );
    let status_output = Command::new("git")
        .args(["status", "--porcelain"])
        .output()
        .context("Failed to check git status")?;

    if status_output.status.success() {
        let status_text = String::from_utf8(status_output.stdout)?;
        if !status_text.trim().is_empty() {
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
                let stdout = String::from_utf8_lossy(&commit_result.stdout);
                eprintln!("🐛 DEBUG: Git commit failed");
                eprintln!("🐛 DEBUG: Stderr: {stderr}");
                eprintln!("🐛 DEBUG: Stdout: {stdout}");
                return Err(anyhow!("Failed to commit changes: {}", stderr));
            }

            // Push to current branch
            eprintln!("🐛 DEBUG: Pushing to branch: {source_branch}");
            let push_result = Command::new("git")
                .args(["push", "origin", &source_branch])
                .output()
                .context("Failed to push changes")?;

            if !push_result.status.success() {
                let stderr = String::from_utf8_lossy(&push_result.stderr);
                eprintln!("🐛 DEBUG: Git push failed");
                eprintln!("🐛 DEBUG: Stderr: {stderr}");
                return Err(anyhow!("Failed to push changes: {}", stderr));
            }

            eprintln!("✅ Changes committed and pushed successfully");
        } else {
            eprintln!("✅ No uncommitted changes found");
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
                "Unknown agent '{}'. Available agents: {:?}",
                agent,
                available_agents
            ));
        }
        config.agents[agent].github_app.clone()
    } else {
        // Use default from config
        config.defaults.docs.github_app.clone()
    };

    // Handle model - use provided value or config default
    let model = arguments
        .get("model")
        .and_then(|v| v.as_str())
        .map(String::from)
        .unwrap_or_else(|| {
            eprintln!(
                "🐛 DEBUG: Using docs default model: {}",
                config.defaults.docs.model
            );
            config.defaults.docs.model.clone()
        });

    // Validate model name (support both Claude API and CLAUDE code formats)
    validate_model_name(&model)?;

    // Task files will be generated by container script from tasks.json

    // Handle include_codebase - use provided value or config default
    let include_codebase = arguments
        .get("include_codebase")
        .and_then(|v| v.as_bool())
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

    eprintln!("🐛 DEBUG: Local working directory: {working_directory}");
    eprintln!("🐛 DEBUG: Container working directory: {container_working_directory}");

    let mut params = vec![
        format!("working-directory={container_working_directory}"),
        format!("repository-url={repository_url}"),
        format!("source-branch={source_branch}"),
        format!("github-app={github_app}"),
        format!("model={model}"),
    ];

    // Always add include_codebase parameter as boolean (required by workflow template)
    params.push(format!("include-codebase={include_codebase}"));

    eprintln!("🐛 DEBUG: Docs workflow submitting with model: {model}");
    eprintln!("🐛 DEBUG: Full Argo parameters: {params:?}");

    let mut args = vec![
        "submit",
        "--from",
        "workflowtemplate/docsrun-template",
        "-n",
        "agent-platform",
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
        Err(e) => Err(anyhow!("Failed to submit docs workflow: {}", e)),
    }
}

#[allow(clippy::disallowed_macros)]
fn handle_play_workflow(arguments: &HashMap<String, Value>) -> Result<Value> {
    let task_id = arguments
        .get("task_id")
        .and_then(|v| v.as_u64())
        .ok_or(anyhow!("Missing required parameter: task_id"))?;

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
            "Invalid service name '{}'. Must contain only lowercase letters, numbers, and hyphens",
            service
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
        .map(String::from)
        .unwrap_or_else(|| {
            eprintln!(
                "🐛 DEBUG: Using play default CLI: {}",
                config.defaults.play.cli
            );
            config.defaults.play.cli.clone()
        });

    // Handle model - use provided value or config default (needed for agent resolution)
    let model = arguments
        .get("model")
        .and_then(|v| v.as_str())
        .map(String::from)
        .unwrap_or_else(|| {
            eprintln!(
                "🐛 DEBUG: Using play default model: {}",
                config.defaults.play.model
            );
            config.defaults.play.model.clone()
        });

    // Handle implementation agent - use provided value or config default
    let implementation_agent_input = arguments
        .get("implementation_agent")
        .and_then(|v| v.as_str())
        .map(String::from)
        .unwrap_or_else(|| config.defaults.play.implementation_agent.clone());

    // Resolve agent name and extract CLI/model if it's a short alias
    let (implementation_agent, implementation_cli, implementation_model) = if let Some(agent_config) = config.agents.get(&implementation_agent_input) {
        // Use the structured agent configuration
        let agent_cli = if agent_config.cli.is_empty() { cli.clone() } else { agent_config.cli.clone() };
        let agent_model = if agent_config.model.is_empty() { model.clone() } else { agent_config.model.clone() };
        (agent_config.github_app.clone(), agent_cli, agent_model)
    } else {
        // Not a configured agent, use provided name with defaults
        (implementation_agent_input, cli.clone(), model.clone())
    };

    // Handle quality agent - use provided value or config default
    let quality_agent_input = arguments
        .get("quality_agent")
        .and_then(|v| v.as_str())
        .map(String::from)
        .unwrap_or_else(|| config.defaults.play.quality_agent.clone());

    // Resolve agent name and extract CLI/model if it's a short alias
    let (quality_agent, quality_cli, quality_model) = if let Some(agent_config) = config.agents.get(&quality_agent_input) {
        // Use the structured agent configuration
        let agent_cli = if agent_config.cli.is_empty() { cli.clone() } else { agent_config.cli.clone() };
        let agent_model = if agent_config.model.is_empty() { model.clone() } else { agent_config.model.clone() };
        (agent_config.github_app.clone(), agent_cli, agent_model)
    } else {
        // Not a configured agent, use provided name with defaults
        (quality_agent_input, cli.clone(), model.clone())
    };

    // Handle testing agent - use provided value or config default
    let testing_agent_input = arguments
        .get("testing_agent")
        .and_then(|v| v.as_str())
        .map(String::from)
        .unwrap_or_else(|| config.defaults.play.testing_agent.clone());

    // Resolve agent name and extract CLI/model if it's a short alias
    let (testing_agent, testing_cli, testing_model) = if let Some(agent_config) = config.agents.get(&testing_agent_input) {
        // Use the structured agent configuration
        let agent_cli = if agent_config.cli.is_empty() { cli.clone() } else { agent_config.cli.clone() };
        let agent_model = if agent_config.model.is_empty() { model.clone() } else { agent_config.model.clone() };
        (agent_config.github_app.clone(), agent_cli, agent_model)
    } else {
        // Not a configured agent, use provided name with defaults
        (testing_agent_input, cli.clone(), model.clone())
    };

    // Validate model name (support both Claude API and CLAUDE code formats)
    validate_model_name(&model)?;

    // Validate agent-specific models
    validate_model_name(&implementation_model)
        .map_err(|e| anyhow!("Invalid implementation agent model: {}", e))?;
    validate_model_name(&quality_model)
        .map_err(|e| anyhow!("Invalid quality agent model: {}", e))?;
    validate_model_name(&testing_model)
        .map_err(|e| anyhow!("Invalid testing agent model: {}", e))?;

    eprintln!("🐛 DEBUG: Play workflow submitting with task_id: {task_id}");
    eprintln!("🐛 DEBUG: Play workflow repository: {repository}");
    eprintln!("🐛 DEBUG: Play workflow service: {service}");
    eprintln!("🐛 DEBUG: Implementation agent: {implementation_agent} (CLI: {implementation_cli}, Model: {implementation_model})");
    eprintln!("🐛 DEBUG: Quality agent: {quality_agent} (CLI: {quality_cli}, Model: {quality_model})");
    eprintln!("🐛 DEBUG: Testing agent: {testing_agent} (CLI: {testing_cli}, Model: {testing_model})");

    // Check for requirements.yaml file
    // Try to determine workspace directory, but don't fail if we can't
    let workspace_dir_result = std::env::var("WORKSPACE_FOLDER_PATHS")
        .map(|paths| {
            let first_path = paths.split(',').next().unwrap_or(&paths).trim();
            std::path::PathBuf::from(first_path)
        })
        .or_else(|_| std::env::current_dir());
    
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
        format!("quality-agent={quality_agent}"),
        format!("quality-cli={quality_cli}"),
        format!("quality-model={quality_model}"),
        format!("testing-agent={testing_agent}"),
        format!("testing-cli={testing_cli}"),
        format!("testing-model={testing_model}"),
        format!("model={model}"),
    ];

    // Load and encode requirements.yaml if it exists
    if let Some(path) = requirements_path {
        let requirements_content = std::fs::read_to_string(&path)
            .context(format!("Failed to read requirements file: {}", path.display()))?;
        
        // Base64 encode the requirements
        use base64::{engine::general_purpose, Engine as _};
        let encoded = general_purpose::STANDARD.encode(requirements_content);
        params.push(format!("task-requirements={encoded}"));
        eprintln!("✅ Encoded requirements.yaml for workflow");
    } else {
        // Always provide task-requirements parameter, even if empty (Argo requires it)
        params.push("task-requirements=".to_string());
    }

    let mut args = vec![
        "submit",
        "--from",
        "workflowtemplate/play-workflow-template",
        "-n",
        "agent-platform",
    ];

    // Add all parameters to the command
    for param in &params {
        args.push("-p");
        args.push(param);
    }

    match run_argo_cli(&args) {
        Ok(output) => Ok(json!({
            "success": true,
            "message": "Play workflow submitted successfully",
            "output": output,
            "task_id": task_id,
            "repository": repository,
            "service": service,
            "docs_repository": docs_repository,
            "docs_project_directory": docs_project_directory,
            "implementation_agent": implementation_agent,
            "quality_agent": quality_agent,
            "testing_agent": testing_agent,
            "model": model,
            "parameters": params
        })),
        Err(e) => Err(anyhow!("Failed to submit play workflow: {}", e)),
    }
}

#[allow(clippy::disallowed_macros)]
fn handle_intake_prd_workflow(arguments: &HashMap<String, Value>) -> Result<Value> {
    eprintln!("🚀 Processing project intake request");

    // Get workspace directory from Cursor environment
    let workspace_dir = std::env::var("WORKSPACE_FOLDER_PATHS")
        .map(|paths| {
            let first_path = paths.split(',').next().unwrap_or(&paths).trim();
            std::path::PathBuf::from(first_path)
        })
        .unwrap_or_else(|_| std::env::current_dir().unwrap_or_default());

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
            "No PRD found. Please create either {}/prd.txt or {}/intake/prd.txt, or provide prd_content parameter",
            project_name,
            project_name
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

    eprintln!("🤖 Using GitHub App: {github_app}");
    eprintln!("🧠 Using Primary Model: {primary_model} ({primary_provider})");
    eprintln!("🔬 Using Research Model: {research_model} ({research_provider})");
    eprintln!("🛡️  Using Fallback Model: {fallback_model} ({fallback_provider})");

    // Create a ConfigMap with the intake files to avoid YAML escaping issues
    let configmap_name = format!(
        "intake-{}-{}",
        project_name.to_lowercase().replace(' ', "-"),
        chrono::Utc::now().timestamp()
    );

    eprintln!("📦 Creating ConfigMap: {configmap_name}");

    // Create ConfigMap with the intake content
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
        "analyze_complexity": analyze_complexity
    });

    // Create the ConfigMap using kubectl
    let cm_output = std::process::Command::new("kubectl")
        .args([
            "create",
            "configmap",
            &configmap_name,
            "-n",
            "agent-platform",
            &format!("--from-literal=prd.txt={prd_content}"),
            &format!("--from-literal=architecture.md={architecture_content}"),
            &format!("--from-literal=config.json={config_json}"),
        ])
        .output();

    if let Err(e) = cm_output {
        return Err(anyhow!("Failed to create ConfigMap: {}", e));
    }

    if let Ok(output) = cm_output {
        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(anyhow!("Failed to create ConfigMap: {}", stderr));
        }
    }

    // Submit Argo workflow with minimal parameters
    let workflow_name = format!("intake-{}", chrono::Utc::now().timestamp());

    let output = std::process::Command::new("argo")
        .args([
            "submit",
            "--from",
            "workflowtemplate/project-intake",
            "-n",
            "agent-platform",
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
                Ok("docs") => Some(handle_docs_workflow(&arguments).map(|result| json!({ 
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
                Ok("intake_prd") => Some(handle_intake_prd_workflow(&arguments).map(|result| json!({ 
                    "content": [{ 
                        "type": "text", 
                        "text": serde_json::to_string_pretty(&result).unwrap_or_else(|_| result.to_string()) 
                    }] 
                }))), 
                Ok("jobs") => Some(handle_jobs_tool(&arguments).map(|result| json!({ 
                    "content": [{ 
                        "type": "text", 
                        "text": serde_json::to_string_pretty(&result).unwrap_or_else(|_| result.to_string()) 
                    }] 
                }))), 
                Ok("stop_job") => Some(handle_stop_job_tool(&arguments).map(|result| json!({ 
                    "content": [{ 
                        "type": "text", 
                        "text": serde_json::to_string_pretty(&result).unwrap_or_else(|_| result.to_string()) 
                    }] 
                }))),
                Ok("docs_ingest") => Some(handle_docs_ingest_tool(&arguments).map(|result| json!({
                    "content": [{
                        "type": "text",
                        "text": result
                    }]
                }))), 
                Ok("input") => Some(handle_send_job_input(&arguments).map(|result| json!({ 
                    "content": [{ 
                        "type": "text", 
                        "text": serde_json::to_string_pretty(&result).unwrap_or_else(|_| result.to_string()) 
                    }] 
                }))), 
                Ok(unknown) => Some(Err(anyhow!("Unknown tool: {}", unknown))), 
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

    Some(Err(anyhow!("Unknown method: {}", method)))
}

fn run_kubectl_json(args: &[&str]) -> Result<Value> {
    let output = std::process::Command::new("kubectl").args(args).output()?;
    if output.status.success() {
        let stdout = String::from_utf8(output.stdout)?;
        let v: Value = serde_json::from_str(&stdout)?;
        Ok(v)
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr);
        Err(anyhow!("kubectl failed: {}", stderr))
    }
}

fn handle_jobs_tool(arguments: &std::collections::HashMap<String, Value>) -> Result<Value> {
    let namespace = arguments
        .get("namespace")
        .and_then(|v| v.as_str())
        .unwrap_or("agent-platform");

    let include = arguments.get("include").and_then(|v| v.as_array());
    let include_play = include.is_none()
        || include
            .unwrap()
            .iter()
            .any(|x| x.as_str() == Some("play"));
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
                    let name = item.get("metadata").and_then(|m| m.get("name")).and_then(|n| n.as_str()).unwrap_or("");
                    let phase = item.get("status").and_then(|s| s.get("phase")).and_then(|p| p.as_str()).unwrap_or("");

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

    Ok(json!({
        "success": true,
        "namespace": namespace,
        "count": jobs.len(),
        "jobs": jobs
    }))
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
        .unwrap_or("agent-platform");

    match job_type {
        "intake" => {
            // Terminate and delete intake workflow
            let _ = run_argo_cli(&["terminate", name, "-n", namespace]);
            match run_argo_cli(&["delete", name, "-n", namespace]) {
                Ok(msg) => Ok(json!({"success": true, "message": format!("Deleted intake workflow {name}: {msg}"), "namespace": namespace})),
                Err(e) => Err(anyhow!(format!("Failed to delete intake workflow {name}: {e}")))
            }
        }
        "play" => {
            // Stop play workflow using Argo CLI
            match run_argo_cli(&["stop", name, "-n", namespace]) {
                Ok(_msg) => Ok(json!({"success": true, "message": format!("Stopped play workflow {name}"), "namespace": namespace})),
                Err(e) => Err(anyhow!(format!("Failed to stop play workflow {name}: {e}")))
            }
        }
        "workflow" => {
            // Stop generic workflow using Argo CLI
            match run_argo_cli(&["stop", name, "-n", namespace]) {
                Ok(_msg) => Ok(json!({"success": true, "message": format!("Stopped workflow {name}"), "namespace": namespace})),
                Err(e) => Err(anyhow!(format!("Failed to stop workflow {name}: {e}")))
            }
        }
        other => Err(anyhow!(format!("Unsupported job_type: {other}. Supported types: intake, play, workflow"))),
    }
}

#[allow(dead_code)]
fn handle_anthropic_message_tool(arguments: &std::collections::HashMap<String, Value>) -> Result<Value> {
    let api_key = std::env::var("ANTHROPIC_API_KEY")
        .map_err(|_| anyhow!("ANTHROPIC_API_KEY environment variable not set"))?;
    let model = arguments
        .get("model")
        .and_then(|v| v.as_str())
        .ok_or(anyhow!("model is required"))?;
    let system = arguments.get("system").and_then(|v| v.as_str());
    let max_tokens = arguments
        .get("max_tokens")
        .and_then(|v| v.as_u64())
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
    if let Some(s) = system { body["system"] = json!(s); }

    let resp = client
        .post("https://api.anthropic.com/v1/messages")
        .header("x-api-key", api_key)
        .header("anthropic-version", "2023-06-01")
        .json(&body)
        .send()
        .and_then(|r| r.error_for_status())
        .map_err(|e| anyhow!(format!("Anthropic request failed: {e}")))?;

    let json_resp: Value = resp.json().map_err(|e| anyhow!(format!("Failed to parse Anthropic response: {e}")))?;
    Ok(json_resp)
}

fn handle_docs_ingest_tool(arguments: &std::collections::HashMap<String, Value>) -> Result<String> {
    let github_url = arguments
        .get("repository_url")
        .and_then(|v| v.as_str())
        .ok_or(anyhow!("repository_url is required"))?;
    
    // Validate it's a GitHub URL
    if !github_url.contains("github.com") {
        return Err(anyhow!("Only GitHub repositories are currently supported. URL must contain 'github.com'"));
    }
    
    // Get configuration from CTO_CONFIG
    let config = CTO_CONFIG.get().ok_or_else(|| anyhow!("CTO configuration not loaded"))?;
    
    let doc_server_url = arguments
        .get("doc_server_url")
        .and_then(|v| v.as_str())
        .unwrap_or(&config.defaults.docs_ingest.doc_server_url);
    
    let doc_type = arguments
        .get("doc_type")
        .and_then(|v| v.as_str())
        .ok_or(anyhow!("doc_type is required"))?;
    
    // Check for ANTHROPIC_API_KEY
    let api_key = std::env::var("ANTHROPIC_API_KEY")
        .map_err(|_| anyhow!("ANTHROPIC_API_KEY environment variable not set"))?;
    
    // Create the Claude prompt for analyzing the repository
    #[allow(clippy::uninlined_format_args)]
    let analysis_prompt = format!(
        r#"Analyze the GitHub repository at {github_url} and determine the optimal documentation ingestion strategy.

You are an expert at identifying and extracting valuable documentation from software repositories.

TASK: Generate a documentation ingestion plan for doc_type '{doc_type}' that will:
1. Clone the repository
2. Extract relevant documentation
3. Ingest it into the doc server at {doc_server_url}

The user has specified that this documentation should be categorized as '{doc_type}' type.

Your response must be a valid JSON object with this exact structure:
{{
  "doc_type": "{doc_type}",
  "include_paths": ["path1", "path2"],
  "exclude_paths": ["test", "vendor"],
  "extensions": ["md", "rst", "html"],
  "reasoning": "Brief explanation here"
}}

IMPORTANT:
- Respond ONLY with the JSON object
- Do not include any text before or after the JSON
- Use the exact doc_type value provided: "{doc_type}"
- Include reasonable defaults if repository structure is unclear"#,
        github_url = github_url,
        doc_type = doc_type,
        doc_server_url = doc_server_url
    );
    
    // Call Claude API to analyze the repository using configured model
    let model = &config.defaults.docs_ingest.model;
    let analysis = call_claude_api(&api_key, &analysis_prompt, model)?;
    
    // Parse the analysis response
    let analysis_text = analysis
        .get("content")
        .and_then(|v| v.as_array())
        .and_then(|arr| arr.first())
        .and_then(|msg| msg.get("text"))
        .and_then(|t| t.as_str())
        .ok_or_else(|| {
            eprintln!("DEBUG: Full Claude API response: {analysis:?}");
            anyhow!("Failed to get Claude analysis response text")
        })?;
    
    // Extract JSON from the response - try direct parse first, then extract
    let strategy_json: Value = match serde_json::from_str(analysis_text) {
        Ok(json) => json,
        Err(_) => {
            // Try to extract JSON object from the response
            let chars: Vec<char> = analysis_text.chars().collect();
            let mut start = None;
            let mut depth = 0;
            let mut in_string = false;
            let mut escape = false;
            let mut json_result = None;
            
            for (i, &ch) in chars.iter().enumerate() {
                if escape {
                    escape = false;
                    continue;
                }
                
                match ch {
                    '\\' if in_string => escape = true,
                    '"' if !escape => in_string = !in_string,
                    '{' if !in_string => {
                        if depth == 0 {
                            start = Some(i);
                        }
                        depth += 1;
                    }
                    '}' if !in_string => {
                        depth -= 1;
                        if depth == 0 && start.is_some() {
                            let json_str = &analysis_text[start.unwrap()..=i];
                            if let Ok(json) = serde_json::from_str::<Value>(json_str) {
                                json_result = Some(json);
                                break;
                            }
                        }
                    }
                    _ => {}
                }
            }
            
            json_result.ok_or_else(|| {
                eprintln!("DEBUG: Could not extract JSON from Claude's response. Full text:");
                eprintln!("{analysis_text}");
                eprintln!("---");
                anyhow!("No valid JSON found in Claude's response. Response length: {} chars", analysis_text.len())
            })?
        }
    };
    
    // Doc type is already known from user input, but verify Claude used it
    let _claude_doc_type = strategy_json
        .get("doc_type")
        .and_then(|v| v.as_str())
        .unwrap_or(doc_type);
    
    let include_paths = strategy_json
        .get("include_paths")
        .and_then(|v| v.as_array())
        .map(|arr| arr.iter()
            .filter_map(|v| v.as_str())
            .collect::<Vec<_>>()
            .join(","))
        .unwrap_or_else(|| "docs/,Documentation/,README.md".to_string());
    
    let extensions = strategy_json
        .get("extensions")
        .and_then(|v| v.as_array())
        .map(|arr| arr.iter()
            .filter_map(|v| v.as_str())
            .collect::<Vec<_>>()
            .join(","))
        .unwrap_or_else(|| "md,rst,html".to_string());
    
    let reasoning = strategy_json
        .get("reasoning")
        .and_then(|v| v.as_str())
        .unwrap_or("No reasoning provided");
    
    // Build the JSON payload with analysis results
    let payload = json!({
        "url": github_url,
        "doc_type": doc_type,
        "include_paths": include_paths,
        "extensions": extensions,
        "yes": true
    });

    // Always execute ingestion automatically
    // Create a temporary file to safely pass JSON payload and avoid shell injection
    let json_payload = serde_json::to_string(&payload)?;

    // Use NamedTempFile for automatic cleanup
    let mut temp_file = NamedTempFile::new()
        .with_context(|| "Failed to create temporary file for JSON payload")?;
    temp_file.write_all(json_payload.as_bytes())
        .with_context(|| "Failed to write JSON payload to temporary file")?;
    temp_file.flush()
        .with_context(|| "Failed to flush temporary file")?;

    let temp_file_path = temp_file.path().to_string_lossy().to_string();
    let _temp_file_guard = temp_file; // Keep alive during execution

    let cmd = format!("curl -s -X POST {}/ingest/intelligent -H 'Content-Type: application/json' -d @{}", doc_server_url, temp_file_path);
    
    let mut output = "📊 Repository Analysis Complete\n\n".to_string();
    output.push_str(&format!("🔗 Repository: {github_url}\n"));
    output.push_str(&format!("📁 Doc Type: {doc_type}\n"));
    output.push_str(&format!("📂 Paths: {include_paths}\n"));
    output.push_str(&format!("📄 Extensions: {extensions}\n"));
    output.push_str(&format!("💭 Reasoning: {reasoning}\n\n"));
    
    output.push_str("🚀 Executing ingestion...\n\n");
    output.push_str(&format!("⚡ Executing:\n{cmd}\n"));

    let result = Command::new("sh")
        .arg("-c")
        .arg(&cmd)
        .output()
        .map_err(|e| anyhow!("Failed to execute command: {}", e))?;

    if result.status.success() {
        let stdout = String::from_utf8_lossy(&result.stdout).to_string();
        output.push_str("✅ Request submitted\n");
        if !stdout.trim().is_empty() {
            // Try to parse job_id for convenience
            if let Ok(val) = serde_json::from_str::<Value>(&stdout) {
                if let Some(job_id) = val.get("job_id").and_then(|v| v.as_str()) {
                    output.push_str(&format!(
                        "🆔 Job ID: {job_id}\n🔍 Check status: {doc_server_url}/ingest/jobs/{job_id}\n"
                    ));
                } else {
                    let trimmed_stdout = stdout.trim();
                    output.push_str(&format!("📤 Response: {trimmed_stdout}\n"));
                }
            } else {
                let trimmed_stdout = stdout.trim();
                output.push_str(&format!("📤 Response: {trimmed_stdout}\n"));
            }
        }
        output.push_str("\n📡 Ingestion running asynchronously. Use the status URL to monitor progress.");
    } else {
        output.push_str(&format!(
            "❌ Request failed: {}\n",
            String::from_utf8_lossy(&result.stderr)
        ));
        return Ok(output);
    }

    Ok(output)
}

fn call_claude_api(api_key: &str, prompt: &str, model: &str) -> Result<Value> {
    let body = json!({
        "model": model,
        "max_tokens": 2000,
        "messages": [
            {
                "role": "user",
                "content": prompt
            }
        ]
    });
    
    let client = reqwest::blocking::Client::new();
    let resp = client
        .post("https://api.anthropic.com/v1/messages")
        .header("x-api-key", api_key)
        .header("anthropic-version", "2023-06-01")
        .header("content-type", "application/json")
        .json(&body)
        .send()
        .and_then(|r| r.error_for_status())
        .map_err(|e| anyhow!("Claude API request failed: {}", e))?;
    
    let json_resp: Value = resp.json()
        .map_err(|e| anyhow!("Failed to parse Claude API response: {}", e))?;
    
    Ok(json_resp)
}

fn handle_send_job_input(arguments: &std::collections::HashMap<String, Value>) -> Result<Value> {
    // Inputs
    let namespace = arguments
        .get("namespace")
        .and_then(|v| v.as_str())
        .unwrap_or("agent-platform");
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
    let services_json = run_kubectl_json(&["get", "services", "-n", namespace, "-l", &selector, "-o", "json"])?.to_string();
    let services: Value = serde_json::from_str(&services_json)?;
    let service_items = services
        .get("items")
        .and_then(|v| v.as_array())
        .ok_or(anyhow!("No services found"))?;

    if service_items.is_empty() {
        return Err(anyhow!(
            "No input bridge services found with selector: {}. Available parameters: job_type={}, user={:?}, service={:?}",
            selector, job_type, user, service
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
        .and_then(|p| p.as_u64())
        .ok_or(anyhow!("Service missing port"))?;

    // Construct the service URL
    let service_url = format!("http://{service_name}.{namespace}.svc.cluster.local:{service_port}/input");

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
        let error_text = response.text().unwrap_or_else(|_| "Unknown error".to_string());
        Err(anyhow!(
            "HTTP request failed with status {}: {}",
            status,
            error_text
        ))
    }
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
    eprintln!("🚀 Starting 5D Labs MCP Server...");

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
