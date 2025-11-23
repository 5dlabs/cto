use anyhow::{anyhow, Context, Result};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::collections::HashMap;
use std::path::Path;
use std::process::Command;
use std::sync::OnceLock;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::runtime::Runtime;
use tokio::signal;
use tokio::time::{timeout, Duration};

mod tools;

// Global configuration loaded once at startup
static CTO_CONFIG: OnceLock<CtoConfig> = OnceLock::new();

#[derive(Debug, Deserialize, Clone)]
struct CtoConfig {
    version: String,
    defaults: WorkflowDefaults,
    agents: HashMap<String, String>,
}

#[derive(Debug, Deserialize, Clone)]
struct WorkflowDefaults {
    docs: DocsDefaults,
    code: CodeDefaults,
    #[serde(default)]
    intake: IntakeDefaults,
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
    model: String,
    #[serde(rename = "githubApp")]
    github_app: String,
    #[serde(rename = "continueSession")]
    continue_session: bool,
    #[serde(rename = "workingDirectory")]
    working_directory: String,
    #[serde(rename = "overwriteMemory")]
    overwrite_memory: bool,
    repository: Option<String>,
    #[serde(rename = "docsRepository")]
    docs_repository: Option<String>,
    #[serde(rename = "docsProjectDirectory")]
    docs_project_directory: Option<String>,
    service: Option<String>,
}

#[derive(Debug, Deserialize, Clone)]
struct IntakeDefaults {
    model: String,
    #[serde(rename = "githubApp")]
    github_app: String,
}

impl Default for IntakeDefaults {
    fn default() -> Self {
        IntakeDefaults {
            model: "claude-3-5-sonnet-20241022".to_string(),
            github_app: "agent-platform".to_string(), // Should be configured in cto-config.json
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
        std::env::current_dir()
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
                    }
                    Err(e) => {
                        eprintln!("⚠️  Failed to parse repository config: {e}");
                    }
                },
                Err(e) => {
                    eprintln!("⚠️  Failed to read repository config: {e}");
                }
            }
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
        std::env::current_dir()
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
        config.agents[agent].clone()
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

    // Validate model name
    if !model.starts_with("claude-") {
        return Err(anyhow!(
            "Invalid model '{}'. Must be a valid Claude model name",
            model
        ));
    }

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
fn handle_task_workflow(arguments: &HashMap<String, Value>) -> Result<Value> {
    let task_id = arguments
        .get("task_id")
        .and_then(|v| v.as_u64())
        .ok_or(anyhow!("Missing required parameter: task_id"))?;

    let config = CTO_CONFIG.get().unwrap();

    // Get workspace directory from Cursor environment
    let workspace_dir = std::env::var("WORKSPACE_FOLDER_PATHS")
        .map(|paths| {
            let first_path = paths.split(',').next().unwrap_or(&paths).trim();
            std::path::PathBuf::from(first_path)
        })
        .unwrap_or_else(|_| std::env::current_dir().unwrap_or_default());

    let service = arguments
        .get("service")
        .and_then(|v| v.as_str())
        .or(config.defaults.code.service.as_deref())
        .ok_or(anyhow!("Missing required parameter: service. Please provide it or set defaults.code.service in config"))?;

    // Handle repository - use provided value or config default
    let repository = arguments
        .get("repository")
        .and_then(|v| v.as_str())
        .map(String::from)
        .or_else(|| config.defaults.code.repository.clone())
        .ok_or(anyhow!("No repository specified. Please provide a 'repository' parameter or set defaults.code.repository in config"))?;

    let docs_project_directory = arguments
        .get("docs_project_directory")
        .and_then(|v| v.as_str())
        .or(config.defaults.code.docs_project_directory.as_deref())
        .ok_or(anyhow!("Missing required parameter: docs_project_directory. Please provide it or set defaults.code.docsProjectDirectory in config"))?;

    // Validate repository URL
    validate_repository_url(&repository)?;

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
        .or_else(|| config.defaults.code.docs_repository.clone())
        .ok_or(anyhow!("No docs_repository specified. Please provide a 'docs_repository' parameter or set defaults.code.docsRepository in config"))?;

    validate_repository_url(&docs_repository)?;

    // Handle working directory - use provided value or config default
    let working_directory = arguments
        .get("working_directory")
        .and_then(|v| v.as_str())
        .unwrap_or(&config.defaults.code.working_directory);

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
        config.agents[agent].clone()
    } else {
        // Use default from config
        config.defaults.code.github_app.clone()
    };

    // Handle model - use provided value or config default
    let model = arguments
        .get("model")
        .and_then(|v| v.as_str())
        .map(String::from)
        .unwrap_or_else(|| {
            eprintln!(
                "🐛 DEBUG: Using code default model: {}",
                config.defaults.code.model
            );
            config.defaults.code.model.clone()
        });

    // Validate model name
    if !model.starts_with("claude-") {
        return Err(anyhow!(
            "Invalid model '{}'. Must be a valid Claude model name",
            model
        ));
    }

    // Auto-detect docs branch (fail if not available, using workspace directory)
    let docs_branch = get_git_current_branch_in_dir(Some(&workspace_dir))
        .context("Failed to auto-detect git branch. Ensure you're in a git repository.")?;

    // Handle continue session - use provided value or config default
    let continue_session = arguments
        .get("continue_session")
        .and_then(|v| v.as_bool())
        .unwrap_or(config.defaults.code.continue_session);

    // Handle overwrite memory - use provided value or config default
    let overwrite_memory = arguments
        .get("overwrite_memory")
        .and_then(|v| v.as_bool())
        .unwrap_or(config.defaults.code.overwrite_memory);

    eprintln!("🐛 DEBUG: Task workflow working directory: {working_directory}");

    let mut params = vec![
        format!("task-id={task_id}"),
        format!("service-id={service}"),
        format!("repository-url={repository}"),
        format!("docs-repository-url={docs_repository}"),
        format!("docs-project-directory={docs_project_directory}"),
        format!("working-directory={working_directory}"),
        format!("github-app={github_app}"),
        format!("model={model}"),
        format!("continue-session={continue_session}"),
        format!("overwrite-memory={overwrite_memory}"),
        format!("docs-branch={docs_branch}"),
        format!("context-version=0"), // Auto-assign by controller
    ];

    // Check for requirements.yaml file - resolve relative to the effective working directory
    // Build effective base directory: join WORKSPACE root with working_directory (unless absolute)
    let working_path = std::path::PathBuf::from(working_directory);
    let base_dir = if working_path.is_absolute() {
        working_path.clone()
    } else {
        workspace_dir.join(working_directory)
    };
    let docs_dir = base_dir.join(docs_project_directory);
    let task_requirements_path = docs_dir.join(format!("task-{task_id}/requirements.yaml"));
    let project_requirements_path = docs_dir.join("requirements.yaml");
    let taskmaster_requirements_path = docs_dir.join(".taskmaster/requirements.yaml");

    eprintln!(
        "🔍 Resolving requirements.yaml under: {} (docs_project_directory='{}')",
        docs_dir.display(),
        docs_project_directory
    );

    let requirements_path = if task_requirements_path.exists() {
        eprintln!("📋 Found task-specific requirements.yaml for task {task_id}");
        task_requirements_path.to_string_lossy().to_string()
    } else if project_requirements_path.exists() {
        eprintln!("📋 Found project-level requirements.yaml for task {task_id}");
        project_requirements_path.to_string_lossy().to_string()
    } else if taskmaster_requirements_path.exists() {
        eprintln!("📋 Found TaskMaster requirements.yaml for task {task_id}");
        taskmaster_requirements_path.to_string_lossy().to_string()
    } else {
        String::new() // No requirements file found
    };

    if !requirements_path.is_empty() {
        let requirements_content = std::fs::read_to_string(&requirements_path).context(format!(
            "Failed to read requirements file: {requirements_path}"
        ))?;

        // Base64 encode the requirements YAML
        use base64::{engine::general_purpose, Engine as _};
        let encoded_requirements =
            general_purpose::STANDARD.encode(requirements_content.as_bytes());
        params.push(format!("task-requirements={encoded_requirements}"));

        eprintln!("✓ Task requirements encoded and added to workflow parameters");
    } else {
        // Always provide task-requirements parameter, even if empty (Argo requires it)
        params.push("task-requirements=".to_string());
        eprintln!("ℹ️ No requirements.yaml found, using empty task-requirements");

        // Fall back to old env/env_from_secrets parameters if provided
        // Handle env object - convert to JSON string for workflow parameter
        if let Some(env) = arguments.get("env").and_then(|v| v.as_object()) {
            let env_json = serde_json::to_string(env)?;
            params.push(format!("env={env_json}"));
        }

        // Handle env_from_secrets array - convert to JSON string for workflow parameter
        if let Some(env_from_secrets) = arguments.get("env_from_secrets").and_then(|v| v.as_array())
        {
            let env_from_secrets_json = serde_json::to_string(env_from_secrets)?;
            params.push(format!("envFromSecrets={env_from_secrets_json}"));
        }
    }

    let mut args = vec![
        "submit",
        "--from",
        "workflowtemplate/coderun-template",
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
            "message": "Task implementation workflow submitted successfully",
            "output": output,
            "task_id": task_id,
            "service": service,
            "repository": repository,
            "docs_repository": docs_repository,
            "docs_project_directory": docs_project_directory,
            "working_directory": working_directory,
            "github_app": github_app,
            "agent": agent_name.unwrap_or("default"),
            "model": model,
            "continue_session": continue_session,
            "overwrite_memory": overwrite_memory,
            "docs_branch": docs_branch,
            "context_version": 0,
            "parameters": params
        })),
        Err(e) => Err(anyhow!("Failed to submit task workflow: {}", e)),
    }
}

#[allow(clippy::disallowed_macros)]
fn handle_intake_workflow(arguments: &HashMap<String, Value>) -> Result<Value> {
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

    // Use configuration values with defaults
    let github_app = &config.defaults.intake.github_app;
    let model = &config.defaults.intake.model;
    let num_tasks = 50; // Standard task count
    let expand_tasks = true; // Always expand for detailed planning
    let analyze_complexity = true; // Always analyze for better breakdown

    eprintln!("🤖 Using GitHub App: {github_app}");
    eprintln!("🧠 Using model: {model}");

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
        "model": model,
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
            &format!("model={model}"),
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
                Ok("task") => Some(handle_task_workflow(&arguments).map(|result| json!({
                    "content": [{
                        "type": "text",
                        "text": serde_json::to_string_pretty(&result).unwrap_or_else(|_| result.to_string())
                    }]
                }))),
                Ok("export") => Some(handle_export_workflow().map(|result| json!({
                    "content": [{
                        "type": "text",
                        "text": result
                    }]
                }))),
                Ok("intake") => Some(handle_intake_workflow(&arguments).map(|result| json!({
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
    let include_code =
        include.is_none() || include.unwrap().iter().any(|x| x.as_str() == Some("code"));
    let include_docs =
        include.is_none() || include.unwrap().iter().any(|x| x.as_str() == Some("docs"));
    let include_intake = include.is_none()
        || include
            .unwrap()
            .iter()
            .any(|x| x.as_str() == Some("intake"));

    let mut jobs: Vec<Value> = Vec::new();

    if include_code {
        if let Ok(list) = run_kubectl_json(&[
            "get",
            "coderuns.agents.platform",
            "-n",
            namespace,
            "-o",
            "json",
        ]) {
            if let Some(items) = list.get("items").and_then(|v| v.as_array()) {
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
                    let message = item
                        .get("status")
                        .and_then(|s| s.get("message"))
                        .and_then(|p| p.as_str());
                    let job_name = item
                        .get("status")
                        .and_then(|s| s.get("jobName"))
                        .and_then(|p| p.as_str());
                    jobs.push(json!({
                        "type": "code",
                        "name": name,
                        "namespace": namespace,
                        "phase": phase,
                        "message": message,
                        "jobName": job_name
                    }));
                }
            }
        }
    }

    if include_docs {
        if let Ok(list) = run_kubectl_json(&[
            "get",
            "docsruns.agents.platform",
            "-n",
            namespace,
            "-o",
            "json",
        ]) {
            if let Some(items) = list.get("items").and_then(|v| v.as_array()) {
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
                    let message = item
                        .get("status")
                        .and_then(|s| s.get("message"))
                        .and_then(|p| p.as_str());
                    let job_name = item
                        .get("status")
                        .and_then(|s| s.get("jobName"))
                        .and_then(|p| p.as_str());
                    jobs.push(json!({
                        "type": "docs",
                        "name": name,
                        "namespace": namespace,
                        "phase": phase,
                        "message": message,
                        "jobName": job_name
                    }));
                }
            }
        }
    }

    if include_intake {
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
                        jobs.push(json!({
                            "type": "intake",
                            "name": name,
                            "namespace": namespace,
                            "phase": phase
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
        "code" => {
            // Use plural, group-qualified CRD to avoid alias issues
            let out = std::process::Command::new("kubectl")
                .args([
                    "delete",
                    "coderuns.agents.platform",
                    name,
                    "-n",
                    namespace,
                    "--wait=false",
                ]) // trigger finalizer cleanup
                .output()
                .context("Failed to execute kubectl delete coderuns.agents.platform")?;
            if out.status.success() {
                Ok(
                    json!({"success": true, "message": format!("Deleted CodeRun {name}"), "namespace": namespace}),
                )
            } else {
                Err(anyhow!(String::from_utf8_lossy(&out.stderr).to_string()))
            }
        }
        "docs" => {
            // Use plural, group-qualified CRD to avoid alias issues
            let out = std::process::Command::new("kubectl")
                .args([
                    "delete",
                    "docsruns.agents.platform",
                    name,
                    "-n",
                    namespace,
                    "--wait=false",
                ]) // trigger finalizer cleanup
                .output()
                .context("Failed to execute kubectl delete docsruns.agents.platform")?;
            if out.status.success() {
                Ok(
                    json!({"success": true, "message": format!("Deleted DocsRun {name}"), "namespace": namespace}),
                )
            } else {
                Err(anyhow!(String::from_utf8_lossy(&out.stderr).to_string()))
            }
        }
        "intake" => {
            // Try to terminate first, then delete
            let _ = run_argo_cli(&["terminate", name, "-n", namespace]);
            match run_argo_cli(&["delete", name, "-n", namespace]) {
                Ok(msg) => Ok(json!({"success": true, "message": msg, "namespace": namespace})),
                Err(e) => Err(anyhow!(format!("Failed to delete workflow {name}: {e}"))),
            }
        }
        other => Err(anyhow!(format!("Unsupported job_type: {other}"))),
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
    if let Some(s) = system {
        body["system"] = json!(s);
    }

    let resp = client
        .post("https://api.anthropic.com/v1/messages")
        .header("x-api-key", api_key)
        .header("anthropic-version", "2023-06-01")
        .json(&body)
        .send()
        .and_then(|r| r.error_for_status())
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
        .unwrap_or("agent-platform");
    let text = arguments
        .get("text")
        .and_then(|v| v.as_str())
        .ok_or(anyhow!("text is required"))?;
    let fifo_path = arguments
        .get("fifo_path")
        .and_then(|v| v.as_str())
        .unwrap_or("/workspace/agent-input.jsonl");

    // Validate fifo_path to prevent command injection: must be an absolute path with safe characters
    // Allowed: a-zA-Z0-9, '/', '-', '_', '.', (no spaces, quotes, semicolons, pipes, etc.)
    if !(fifo_path.starts_with('/')
        && fifo_path.len() <= 256
        && fifo_path
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '/' || c == '-' || c == '_' || c == '.'))
    {
        return Err(anyhow!(
            "Invalid fifo_path; must be absolute and contain only [a-zA-Z0-9/_-.]"
        ));
    }

    // Routing: explicit name + job_type, or by user label (optionally filtered by job_type)
    let job_type = arguments.get("job_type").and_then(|v| v.as_str());
    let name = arguments.get("name").and_then(|v| v.as_str());
    let user = arguments.get("user").and_then(|v| v.as_str());

    let (selector, pod_ns) = if let (Some(jt), Some(nm)) = (job_type, name) {
        // route by explicit resource name
        let label_selector = match jt {
            "code" => format!("agents.platform/CodeRun={nm}"),
            "docs" => format!("agents.platform/DocsRun={nm}"),
            other => return Err(anyhow!(format!("Unsupported job_type: {other}"))),
        };
        (label_selector, namespace)
    } else if let Some(user_label) = user {
        // route by user label (with optional job_type filter)
        let mut label_selector = format!("agents.platform/user={user_label}");
        if let Some(jt) = job_type {
            label_selector.push(',');
            label_selector.push_str(&format!("agents.platform/jobType={jt}"));
        }
        (label_selector, namespace)
    } else {
        return Err(anyhow!(
            "Provide either (job_type + name) or user label for routing"
        ));
    };

    // Find pod by labels
    let pods_json =
        run_kubectl_json(&["get", "pods", "-n", pod_ns, "-l", &selector, "-o", "json"])?
            .to_string();
    let pods: Value = serde_json::from_str(&pods_json)?;
    let pod_name = pods
        .get("items")
        .and_then(|v| v.as_array())
        .and_then(|arr| arr.first())
        .and_then(|item| item.get("metadata"))
        .and_then(|m| m.get("name"))
        .and_then(|n| n.as_str())
        .ok_or(anyhow!("No running pod found for routing selector"))?;

    // Format a stream-json user message line (one JSON object per line)
    let line = json!({
        "type": "user",
        "message": {
            "role": "user",
            "content": [ { "type": "text", "text": text } ]
        }
    })
    .to_string();

    // Stream input safely using tee without invoking a shell; pass fifo_path as an argument
    // This avoids shell interpolation entirely
    let mut child = std::process::Command::new("kubectl")
        .args([
            "-n", namespace, "exec", "-i", pod_name, "--", "tee", "-a", fifo_path,
        ])
        .env(
            "KUBECONFIG",
            std::env::var("KUBECONFIG").unwrap_or_default(),
        )
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .spawn()
        .context("Failed to exec into pod")?;
    if let Some(stdin) = child.stdin.as_mut() {
        use std::io::Write as _;
        stdin
            .write_all(format!("{line}\n").as_bytes())
            .context("Failed writing input to kubectl exec")?;
    }
    let out = child.wait_with_output()?;
    if out.status.success() {
        Ok(json!({"success": true, "pod": pod_name, "fifo": fifo_path}))
    } else {
        Err(anyhow!(String::from_utf8_lossy(&out.stderr).to_string()))
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

/// Handle export workflow - convert current directory's Rust code to markdown
#[allow(clippy::disallowed_macros)]
fn handle_export_workflow() -> Result<String> {
    // Use WORKSPACE_FOLDER_PATHS to get the actual workspace directory
    let project_dir = std::env::var("WORKSPACE_FOLDER_PATHS")
        .map(|paths| {
            // WORKSPACE_FOLDER_PATHS might contain multiple paths separated by some delimiter
            // For now, take the first one (or the only one)
            let first_path = paths.split(',').next().unwrap_or(&paths).trim();
            first_path.to_string()
        })
        .map(std::path::PathBuf::from)
        .unwrap_or_else(|_| std::env::current_dir().unwrap_or_default());

    eprintln!("🔍 Using workspace directory: {}", project_dir.display());

    // Create .taskmaster/docs directory if it doesn't exist
    let taskmaster_dir = project_dir.join(".taskmaster");
    let docs_dir = taskmaster_dir.join("docs");

    eprintln!("📁 Creating directory: {}", docs_dir.display());
    eprintln!("📁 Project dir exists: {}", project_dir.exists());
    eprintln!("📁 Project dir is_dir: {}", project_dir.is_dir());

    std::fs::create_dir_all(&docs_dir).with_context(|| {
        format!(
            "Failed to create .taskmaster/docs directory at: {}",
            docs_dir.display()
        )
    })?;

    let output_file = docs_dir.join("codebase.md");

    // Generate markdown content
    let markdown_content =
        generate_codebase_markdown(&project_dir).context("Failed to generate codebase markdown")?;

    // Write to file
    std::fs::write(&output_file, &markdown_content).context("Failed to write codebase.md")?;

    Ok(format!(
        "✅ Exported codebase to: {}",
        output_file.display()
    ))
}

/// Generate markdown representation of Rust codebase
fn generate_codebase_markdown(project_dir: &std::path::Path) -> Result<String> {
    let mut markdown = String::new();

    // Add header
    let project_name = project_dir
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("Unknown Project");

    markdown.push_str(&format!("# Project: {project_name}\n\n"));

    // Read Cargo.toml if it exists
    let cargo_toml_path = project_dir.join("Cargo.toml");
    if cargo_toml_path.exists() {
        if let Ok(cargo_content) = std::fs::read_to_string(&cargo_toml_path) {
            markdown.push_str("## Cargo.toml\n\n```toml\n");
            markdown.push_str(&cargo_content);
            markdown.push_str("\n```\n\n");
        }
    }

    // Find and process all relevant source files
    markdown.push_str("## Source Files\n\n");

    process_source_files(&mut markdown, project_dir, project_dir)?;

    Ok(markdown)
}

/// Recursively process source files
fn process_source_files(
    markdown: &mut String,
    current_dir: &std::path::Path,
    project_root: &std::path::Path,
) -> Result<()> {
    let entries = std::fs::read_dir(current_dir).context("Failed to read directory")?;

    for entry in entries {
        let entry = entry.context("Failed to read directory entry")?;
        let path = entry.path();

        // Skip target directory and hidden directories
        if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
            if name == "target" || name.starts_with('.') {
                continue;
            }
        }

        if path.is_dir() {
            process_source_files(markdown, &path, project_root)?;
        } else if let Some(ext) = path.extension().and_then(|s| s.to_str()) {
            // Include multiple file types beyond just .rs
            let (language, should_include) = match ext {
                "rs" => ("rust", true),
                "py" => ("python", true),
                "sql" => ("sql", true),
                "toml" => ("toml", true),
                "yml" | "yaml" => ("yaml", true),
                "json" => ("json", true),
                "md" => ("markdown", true),
                "txt" => ("text", true),
                "sh" => ("bash", true),
                "dockerfile" => ("dockerfile", true),
                _ => ("text", false),
            };

            // Also include files without extensions but with specific names
            let should_include = should_include
                || matches!(
                    path.file_name().and_then(|n| n.to_str()),
                    Some("Dockerfile") | Some("README") | Some("LICENSE")
                );

            if should_include {
                // Get relative path from project root
                let relative_path = path
                    .strip_prefix(project_root)
                    .context("Failed to get relative path")?;

                markdown.push_str(&format!("### {}\n\n", relative_path.display()));

                match std::fs::read_to_string(&path) {
                    Ok(content) => {
                        markdown.push_str(&format!("```{language}\n"));
                        markdown.push_str(&content);
                        markdown.push_str("\n```\n\n");
                    }
                    Err(e) => {
                        markdown.push_str(&format!("*Error reading file: {e}*\n\n"));
                    }
                }
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
