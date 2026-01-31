//! Linear Sidecar Binary
//!
//! Receives agent logs from FluentD and streams them to Linear's agent dialog.
//! Supports multiple log sources:
//!   - HTTP ingest (FluentD forward)
//!   - File watching (JSONL stream)
//!
//! Environment:
//!   LOG_SOURCE: "http" | "file" (default: "file")
//!   HTTP_PORT: Port for HTTP ingest (default: 8080)
//!   STREAM_FILE: Path to JSONL stream file
//!   LINEAR_OAUTH_TOKEN: Linear API token
//!   LINEAR_ISSUE_IDENTIFIER: Issue to post to (e.g., "CTO-123")
//!   CTO_AGENT_NAME: Agent name for config lookup
//!   CTO_CONFIG_PATH: Path to cto-config.json
//!   CTO_SKILLS_DIR: Path to skills directory
//!   WORKSPACE_PATH: Path to agent workspace (for reading task context)
//!   TASK_PROMPT_PATH: Path to task prompt file (default: $WORKSPACE_PATH/prompt.md)
//!   TASK_ACCEPTANCE_PATH: Path to acceptance criteria (default: $WORKSPACE_PATH/acceptance-criteria.md)

use anyhow::{Context, Result};
use axum::{
    extract::State,
    http::StatusCode,
    routing::{get, post},
    Json, Router,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::env;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, error, info, warn};

// =============================================================================
// Types
// =============================================================================

/// Log entry from FluentD or JSONL file
/// Supports Claude CLI stream-json format:
///   - Init: {"type":"system","subtype":"init","model":"...","tools":[...],"skills":[...]}
///   - Assistant: {"type":"assistant","message":{"content":[{"type":"text","text":"..."}]}}
///   - Tool use: {"type":"assistant","message":{"content":[{"type":"tool_use","name":"...","input":{}}]}}
///   - User: {"type":"user","message":{"content":[{"type":"tool_result","content":"..."}]}}
#[derive(Debug, Clone, Deserialize, Serialize)]
struct LogEntry {
    #[serde(rename = "type", default)]
    entry_type: Option<String>,
    
    // Claude-specific: subtype for system events
    #[serde(default)]
    subtype: Option<String>,
    
    // Common fields
    #[serde(default)]
    message: Option<serde_json::Value>,  // Can be string or object (Claude uses object)
    #[serde(default)]
    content: Option<String>,
    
    // Tool call fields
    #[serde(default)]
    tool_name: Option<String>,
    #[serde(default)]
    name: Option<String>,
    #[serde(default)]
    tool_input: Option<serde_json::Value>,
    #[serde(default)]
    input: Option<serde_json::Value>,
    
    // Init fields (Claude CLI)
    #[serde(default)]
    model: Option<String>,
    #[serde(default)]
    tools: Option<Vec<String>>,
    #[serde(default)]
    skills: Option<Vec<String>>,
    #[serde(default)]
    mcp_servers: Option<serde_json::Value>,  // Can be array of objects or strings
    
    // Session tracking
    #[serde(default)]
    session_id: Option<String>,
    #[serde(default)]
    uuid: Option<String>,
    
    // FluentD metadata
    #[serde(default)]
    cli: Option<String>,
    #[serde(default)]
    agent: Option<String>,
    #[serde(default)]
    container_name: Option<String>,
    #[serde(default)]
    timestamp: Option<String>,
    
    // Catch-all for other fields
    #[serde(flatten)]
    extra: HashMap<String, serde_json::Value>,
}

/// Helper to check if this is a Claude init event
fn is_claude_init(entry: &LogEntry) -> bool {
    // Claude format: {"type":"system","subtype":"init",...}
    entry.entry_type.as_deref() == Some("system") && entry.subtype.as_deref() == Some("init")
}

/// Helper to extract text content from Claude assistant message
fn extract_claude_text(entry: &LogEntry) -> Option<String> {
    let msg = entry.message.as_ref()?;
    let content = msg.get("content")?.as_array()?;
    for item in content {
        if item.get("type")?.as_str()? == "text" {
            return item.get("text")?.as_str().map(|s| s.to_string());
        }
    }
    None
}

/// Helper to extract tool use from Claude assistant message
fn extract_claude_tool_use(entry: &LogEntry) -> Option<(String, serde_json::Value)> {
    let msg = entry.message.as_ref()?;
    let content = msg.get("content")?.as_array()?;
    for item in content {
        if item.get("type")?.as_str()? == "tool_use" {
            let name = item.get("name")?.as_str()?.to_string();
            let input = item.get("input").cloned().unwrap_or(serde_json::json!({}));
            return Some((name, input));
        }
    }
    None
}

/// Parsed task context from prompt.md
#[derive(Debug, Clone, Default)]
struct TaskContext {
    title: Option<String>,
    goal: Option<String>,
    role: Option<String>,
    agent: Option<String>,
    language: Option<String>,
    // Acceptance criteria tracking
    total_criteria: usize,
    completed_criteria: usize,
}

/// Get emoji for agent based on name
fn get_agent_emoji(agent_name: &str) -> &'static str {
    match agent_name.to_lowercase().as_str() {
        // Primary CTO agents
        "bolt" => "⚡",
        "rex" => "🦖",
        "morgan" => "🧙",
        "blaze" => "🔥",
        // Database deployers
        "postgres-deployer" | "postgresql" | "postgres" => "🐘",
        "mongo-deployer" | "mongodb" | "mongo" => "🍃",
        "redis-deployer" | "redis" => "🔴",
        // Infrastructure agents
        "kafka-deployer" | "kafka" => "📨",
        "seaweedfs-deployer" | "seaweedfs" => "🌊",
        "rabbitmq-deployer" | "rabbitmq" => "🐰",
        // Security & networking
        "security-agent" | "security" => "🔐",
        "network-agent" | "networking" => "🌐",
        // Default
        _ => "🤖"
    }
}

/// Parse task context from prompt.md content
fn parse_task_context(content: &str) -> TaskContext {
    let mut ctx = TaskContext::default();
    let lines: Vec<&str> = content.lines().collect();
    
    // Extract title from first # heading
    for line in &lines {
        if line.starts_with("# ") {
            ctx.title = Some(line[2..].trim().to_string());
            break;
        }
    }
    
    // Extract metadata from **Agent**: bolt | **Language**: yaml pattern (markdown format)
    for line in &lines {
        if line.contains("**Agent**:") {
            if let Some(agent_part) = line.split("**Agent**:").nth(1) {
                let agent = agent_part.split('|').next()
                    .map(|s| s.trim().to_string());
                ctx.agent = agent;
            }
        }
        if line.contains("**Language**:") {
            if let Some(lang_part) = line.split("**Language**:").nth(1) {
                let lang = lang_part.split('|').next()
                    .map(|s| s.trim().to_string());
                ctx.language = lang;
            }
        }
    }
    
    // Extract metadata from XML tags (new format): <agent>, <objective>
    // Parse <agent> tag
    if let Some(start) = content.find("<agent>") {
        if let Some(end) = content.find("</agent>") {
            let agent = content[start + 7..end].trim().to_string();
            if !agent.is_empty() {
                ctx.agent = Some(agent);
            }
        }
    }
    
    // Parse <objective> as goal if not already set
    if ctx.goal.is_none() {
        if let Some(start) = content.find("<objective>") {
            if let Some(end) = content.find("</objective>") {
                let objective = content[start + 11..end].trim().to_string();
                if !objective.is_empty() {
                    ctx.goal = Some(objective);
                }
            }
        }
    }
    
    // Extract sections (## Role, ## Goal)
    let mut current_section: Option<&str> = None;
    let mut section_content = String::new();
    
    for line in &lines {
        if line.starts_with("## ") {
            // Save previous section
            if let Some(section) = current_section {
                let content = section_content.trim().to_string();
                match section {
                    "Role" => ctx.role = Some(content),
                    "Goal" => ctx.goal = Some(content),
                    _ => {}
                }
            }
            // Start new section
            current_section = Some(line[3..].trim());
            section_content = String::new();
        } else if current_section.is_some() {
            section_content.push_str(line);
            section_content.push('\n');
        }
    }
    
    // Save final section
    if let Some(section) = current_section {
        let content = section_content.trim().to_string();
        match section {
            "Role" => ctx.role = Some(content),
            "Goal" => ctx.goal = Some(content),
            _ => {}
        }
    }
    
    ctx
}

/// Parse acceptance criteria from markdown file
/// Returns (total_criteria, completed_criteria)
fn parse_acceptance_criteria(content: &str) -> (usize, usize) {
    let mut total = 0;
    let mut completed = 0;
    
    for line in content.lines() {
        let trimmed = line.trim();
        // Match checked boxes: - [x] or - [X] or * [x] etc
        if trimmed.starts_with("- [x]") || trimmed.starts_with("- [X]") 
           || trimmed.starts_with("* [x]") || trimmed.starts_with("* [X]") {
            total += 1;
            completed += 1;
        }
        // Match unchecked boxes: - [ ] or * [ ]
        else if trimmed.starts_with("- [ ]") || trimmed.starts_with("* [ ]") {
            total += 1;
        }
    }
    
    (total, completed)
}

// =============================================================================
// Narrative System - Transform tool calls into human-readable journey
// =============================================================================

/// Phases of work in a task journey
#[derive(Debug, Clone, PartialEq, Default)]
enum NarrativePhase {
    #[default]
    Starting,           // Just started
    Understanding,      // Reading PRD, prompt, architecture docs
    Planning,           // Reading subtasks, acceptance criteria  
    Building,           // Writing code, configs, manifests
    Testing,            // Running tests, validation
    Shipping,           // Git operations, PR creation
    Complete,           // Task finished
}

impl NarrativePhase {
    fn emoji(&self) -> &str {
        match self {
            NarrativePhase::Starting => "🎬",
            NarrativePhase::Understanding => "📖",
            NarrativePhase::Planning => "📋",
            NarrativePhase::Building => "🔨",
            NarrativePhase::Testing => "🧪",
            NarrativePhase::Shipping => "🚀",
            NarrativePhase::Complete => "🎉",
        }
    }
    
    fn description(&self) -> &str {
        match self {
            NarrativePhase::Starting => "Getting started...",
            NarrativePhase::Understanding => "Understanding the mission",
            NarrativePhase::Planning => "Planning the work",
            NarrativePhase::Building => "Building",
            NarrativePhase::Testing => "Testing & validating",
            NarrativePhase::Shipping => "Preparing to ship",
            NarrativePhase::Complete => "Mission complete!",
        }
    }
}

/// State for tracking narrative progression
#[derive(Debug, Default, Clone)]
struct NarrativeState {
    phase: NarrativePhase,
    current_subtask: Option<String>,
    subtask_number: u32,
    actions_in_phase: u32,
    last_action_summary: Option<String>,
    milestones: Vec<String>,
}

/// Convert a tool call into a narrative description
/// Returns (phase, description, is_significant)
/// is_significant means we should post this to Linear (not just log)
fn narrate_tool_call(tool_name: &str, params: &serde_json::Value, state: &NarrativeState) -> Option<(NarrativePhase, String, bool)> {
    let file_path = params.get("file_path")
        .or_else(|| params.get("path"))
        .and_then(|v| v.as_str())
        .unwrap_or("");
    
    let command = params.get("command")
        .and_then(|v| v.as_str())
        .unwrap_or("");
    
    match tool_name {
        "Read" => {
            // Determine phase and description based on what's being read
            if file_path.contains("prd") || file_path.contains("PRD") {
                Some((NarrativePhase::Understanding, "Checking the PRD to understand what we're building...".to_string(), true))
            } else if file_path.contains("architecture") || file_path.contains("ARCHITECTURE") {
                Some((NarrativePhase::Understanding, "Looking at the architecture to see how things fit together...".to_string(), true))
            } else if file_path.contains("prompt.md") && !file_path.contains("subtask") {
                Some((NarrativePhase::Understanding, "Got the task brief, let me see what's needed...".to_string(), true))
            } else if file_path.contains("acceptance") {
                Some((NarrativePhase::Planning, "Checking the acceptance criteria - need to know what 'done' looks like...".to_string(), true))
            } else if file_path.contains("subtask") && file_path.contains("prompt") {
                // Extract subtask number from path
                let subtask_num = file_path.split('/').rev()
                    .find(|p| p.starts_with("task-") || p.starts_with("subtask"))
                    .and_then(|s| s.split('-').last())
                    .unwrap_or("next");
                Some((NarrativePhase::Planning, format!("Looking at subtask {}...", subtask_num), true))
            } else if file_path.contains("COMPLETION") || file_path.contains("SUMMARY") {
                Some((NarrativePhase::Building, "Checking what's already been done...".to_string(), true))
            } else if file_path.ends_with(".yaml") || file_path.ends_with(".yml") {
                let filename = file_path.split('/').last().unwrap_or("config");
                Some((NarrativePhase::Building, format!("Reading `{}`...", filename), false))
            } else {
                None
            }
        }
        
        "Write" => {
            let filename = file_path.split('/').last().unwrap_or("file");
            if file_path.contains("COMPLETION") || file_path.contains("SUMMARY") {
                Some((NarrativePhase::Building, format!("Writing up the completion report..."), true))
            } else if file_path.ends_with(".yaml") || file_path.ends_with(".yml") {
                // Be specific about what kind of yaml
                if file_path.contains("postgres") {
                    Some((NarrativePhase::Building, "Setting up PostgreSQL cluster config...".to_string(), true))
                } else if file_path.contains("redis") || file_path.contains("valkey") {
                    Some((NarrativePhase::Building, "Configuring Redis for caching...".to_string(), true))
                } else if file_path.contains("kafka") {
                    Some((NarrativePhase::Building, "Wiring up Kafka cluster...".to_string(), true))
                } else if file_path.contains("mongo") {
                    Some((NarrativePhase::Building, "Setting up MongoDB...".to_string(), true))
                } else if file_path.contains("rabbit") {
                    Some((NarrativePhase::Building, "Deploying RabbitMQ...".to_string(), true))
                } else {
                    Some((NarrativePhase::Building, format!("Creating `{}`...", filename), true))
                }
            } else if file_path.ends_with(".go") {
                Some((NarrativePhase::Building, format!("Writing Go code: `{}`", filename), true))
            } else if file_path.ends_with(".rs") {
                Some((NarrativePhase::Building, format!("Writing Rust: `{}`", filename), true))
            } else if file_path.ends_with(".py") {
                Some((NarrativePhase::Building, format!("Writing Python: `{}`", filename), true))
            } else if file_path.ends_with(".ts") || file_path.ends_with(".tsx") {
                Some((NarrativePhase::Building, format!("Writing TypeScript: `{}`", filename), true))
            } else if file_path.contains("test") {
                Some((NarrativePhase::Testing, format!("Adding test: `{}`", filename), true))
            } else {
                Some((NarrativePhase::Building, format!("Creating `{}`", filename), true))
            }
        }
        
        "Edit" => {
            let filename = file_path.split('/').last().unwrap_or("file");
            Some((NarrativePhase::Building, format!("Updating `{}`...", filename), true))
        }
        
        "Bash" => {
            // Categorize bash commands - be specific
            if command.contains("git push") {
                Some((NarrativePhase::Shipping, "Pushing changes to remote...".to_string(), true))
            } else if command.contains("git commit") {
                Some((NarrativePhase::Shipping, "Committing changes...".to_string(), true))
            } else if command.contains("git checkout") || command.contains("git branch") {
                Some((NarrativePhase::Shipping, "Setting up the feature branch...".to_string(), true))
            } else if command.contains("helm lint") {
                Some((NarrativePhase::Testing, "Validating Helm charts...".to_string(), true))
            } else if command.contains("helm template") {
                Some((NarrativePhase::Testing, "Testing Helm template rendering...".to_string(), true))
            } else if command.contains("kubectl") && command.contains("dry-run") {
                Some((NarrativePhase::Testing, "Dry-run testing K8s manifests...".to_string(), true))
            } else if command.contains("yamllint") {
                Some((NarrativePhase::Testing, "Checking YAML formatting...".to_string(), true))
            } else if command.contains("test") || command.contains("lint") || command.contains("check") {
                Some((NarrativePhase::Testing, "Running validation...".to_string(), true))
            } else if command.contains("find") || command.contains("ls") || command.contains("cat") {
                None // Exploratory, skip
            } else {
                None
            }
        }
        
        // MCP tool calls
        tool if tool.starts_with("mcp__") || tool.contains("github") => {
            if tool.contains("create_pull_request") {
                Some((NarrativePhase::Shipping, "🎁 Creating pull request...".to_string(), true))
            } else if tool.contains("create_branch") {
                Some((NarrativePhase::Shipping, "🌿 Creating feature branch...".to_string(), true))
            } else if tool.contains("push") {
                Some((NarrativePhase::Shipping, "📤 Pushing changes...".to_string(), true))
            } else if tool.contains("search") || tool.contains("get") {
                Some((NarrativePhase::Understanding, "🔍 Researching...".to_string(), false)) // Too noisy
            } else {
                None
            }
        }
        
        "Task" | "TaskOutput" => {
            // Subagent spawn
            Some((NarrativePhase::Building, "🤖 Delegating to subagent...".to_string(), true))
        }
        
        _ => None
    }
}

/// Generate a milestone message when entering a new subtask
fn narrate_subtask_start(subtask_name: &str, subtask_num: u32) -> String {
    format!(
        "\n━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n🔨 Working on: Subtask {}.{} - {}\n━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━",
        subtask_num / 10, subtask_num % 10, subtask_name
    )
}

/// Generate a phase transition message
fn narrate_phase_change(old_phase: &NarrativePhase, new_phase: &NarrativePhase) -> Option<String> {
    if old_phase == new_phase {
        return None;
    }
    
    Some(format!("{} {}", new_phase.emoji(), new_phase.description()))
}

/// Agent session state
#[derive(Debug, Default)]
struct SessionState {
    session_id: Option<String>,
    model: Option<String>,
    tools: Vec<String>,
    skills: Vec<String>,
    tools_used: Vec<String>,
    total_entries: u64,
    errors: u64,
    // Work tracking
    files_created: Vec<String>,
    files_modified: Vec<String>,
    subagents_spawned: Vec<String>,
    start_time: Option<std::time::Instant>,
    // Narrative state
    narrative: NarrativeState,
}

/// Shared application state
struct AppState {
    session: RwLock<SessionState>,
    linear_token: String,
    issue_identifier: String,
    agent_name: String,
    // Task context
    task_context: TaskContext,
    workspace_path: Option<PathBuf>,
}

// =============================================================================
// Linear GraphQL Client
// =============================================================================

const LINEAR_API_URL: &str = "https://api.linear.app/graphql";

/// Resolve issue identifier (e.g., "CTOPA-123") to issue UUID
async fn resolve_issue_id(token: &str, identifier: &str) -> Result<String> {
    let client = reqwest::Client::new();
    
    let query = r#"
        query GetIssue($identifier: String!) {
            issue(id: $identifier) {
                id
            }
        }
    "#;
    
    let response = client
        .post(LINEAR_API_URL)
        .header("Authorization", token)
        .header("Content-Type", "application/json")
        .json(&serde_json::json!({
            "query": query,
            "variables": { "identifier": identifier }
        }))
        .send()
        .await
        .context("Failed to resolve issue ID")?;
    
    let body: serde_json::Value = response.json().await?;
    
    if let Some(errors) = body.get("errors") {
        error!("Linear API errors resolving issue: {:?}", errors);
        anyhow::bail!("Failed to resolve issue: {:?}", errors);
    }
    
    let issue_id = body["data"]["issue"]["id"]
        .as_str()
        .context("Issue not found")?
        .to_string();
    
    info!("Resolved {} to {}", identifier, issue_id);
    Ok(issue_id)
}

/// Post a milestone comment on the issue itself (permanent, not in agent dialog)
/// Used for major events: task start, deployments, PR creation, task completion
async fn post_milestone_comment(state: &AppState, milestone_type: MilestoneType, details: &str) -> Result<()> {
    let client = reqwest::Client::new();
    
    // Resolve issue ID first
    let issue_id = resolve_issue_id(&state.linear_token, &state.issue_identifier).await?;
    
    let query = r#"
        mutation CreateComment($input: CommentCreateInput!) {
            commentCreate(input: $input) {
                success
                comment {
                    id
                }
            }
        }
    "#;
    
    // Format milestone message based on type
    let task = &state.task_context;
    let agent_name = task.agent.as_deref().unwrap_or("Agent");
    let agent_emoji = get_agent_emoji(agent_name);
    
    let body = match milestone_type {
        MilestoneType::TaskStarted => {
            let title = task.title.as_deref().unwrap_or("Task");
            format!(
                "{} **{} started working**\n\n📋 {}\n\n{}",
                agent_emoji, agent_name.to_uppercase(), title, details
            )
        }
        MilestoneType::Deployment { ref url } => {
            format!(
                "✅ **Deployed!**\n\n🔗 {}\n\n{}",
                url, details
            )
        }
        MilestoneType::PullRequest { ref url, ref title } => {
            format!(
                "🔗 **Pull Request Ready**\n\n[{}]({})\n\n{}",
                title, url, details
            )
        }
        MilestoneType::TaskCompleted { duration_secs, cost_usd } => {
            let title = task.title.as_deref().unwrap_or("Task");
            let duration_str = if duration_secs >= 60.0 {
                format!("{:.0}m {:.0}s", (duration_secs / 60.0).floor(), duration_secs % 60.0)
            } else {
                format!("{:.1}s", duration_secs)
            };
            format!(
                "🎉 **{} completed {}!**\n\n⏱️ {} │ 💰 ${:.4}\n\n{}",
                agent_name.to_uppercase(), title, duration_str, cost_usd, details
            )
        }
        MilestoneType::Error { ref message } => {
            format!(
                "❌ **Error**\n\n{}\n\n{}",
                message, details
            )
        }
    };
    
    let variables = serde_json::json!({
        "input": {
            "issueId": issue_id,
            "body": body
        }
    });
    
    let response = client
        .post(LINEAR_API_URL)
        .header("Authorization", &state.linear_token)
        .header("Content-Type", "application/json")
        .json(&serde_json::json!({
            "query": query,
            "variables": variables
        }))
        .send()
        .await?;
    
    let status = response.status();
    if !status.is_success() {
        let response_body = response.text().await.unwrap_or_default();
        warn!("Failed to post milestone comment ({}): {}", status, response_body);
    } else {
        info!("📌 Posted milestone comment to issue");
    }
    
    Ok(())
}

/// Types of milestones that get posted as issue comments
#[derive(Debug)]
enum MilestoneType {
    TaskStarted,
    Deployment { url: String },
    PullRequest { url: String, title: String },
    TaskCompleted { duration_secs: f64, cost_usd: f64 },
    Error { message: String },
}

/// Create agent session on Linear issue
/// Note: Linear's AgentSessionCreateOnIssue now only accepts issueId
/// Model, tools, skills are added via activities
/// Post initialization activity showing task context, model, tools, skills
async fn post_init_activity(state: &AppState, session_id: &str, model: &str, tools: &[String], skills: &[String]) -> Result<()> {
    let client = reqwest::Client::new();
    
    let query = r#"
        mutation AddAgentActivity($input: AgentActivityCreateInput!) {
            agentActivityCreate(input: $input) {
                success
            }
        }
    "#;
    
    // Build beautiful init summary
    let task = &state.task_context;
    
    // Header with agent personality
    let agent_name = task.agent.as_deref().unwrap_or("Agent");
    let agent_emoji = get_agent_emoji(agent_name);
    
    let mut body = format!("{} **{} clocked in!**\n\n", agent_emoji, agent_name.to_uppercase());
    
    // Mission brief - the what
    if let Some(ref title) = task.title {
        body.push_str(&format!("📋 **Mission:** {}\n", title));
    }
    if let Some(ref goal) = task.goal {
        let goal_short = if goal.len() > 150 { format!("{}...", &goal[..150]) } else { goal.clone() };
        body.push_str(&format!("🎯 {}\n", goal_short));
    }
    
    body.push_str("\n");
    
    // Equipment loadout - model, tools, skills
    body.push_str("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n");
    body.push_str(&format!("🧠 **Model:** `{}`\n", model));
    
    // MCP Tools - clean display
    let mcp_tools: Vec<_> = tools.iter()
        .filter(|t| t.starts_with("mcp__"))
        .map(|t| t.strip_prefix("mcp__cto-tools__").unwrap_or(t).to_string())
        .collect();
    
    if !mcp_tools.is_empty() {
        let preview: Vec<_> = mcp_tools.iter().take(5).cloned().collect();
        let extras = if mcp_tools.len() > 5 { format!(" +{} more", mcp_tools.len() - 5) } else { String::new() };
        body.push_str(&format!("🔧 **Tools ({}):** {}{}\n", mcp_tools.len(), preview.join(", "), extras));
    }
    
    if !skills.is_empty() {
        let preview: Vec<_> = skills.iter().take(5).cloned().collect();
        let extras = if skills.len() > 5 { format!(" +{} more", skills.len() - 5) } else { String::new() };
        body.push_str(&format!("📚 **Skills ({}):** {}{}\n", skills.len(), preview.join(", "), extras));
    }
    
    // Acceptance criteria as progress bar
    if task.total_criteria > 0 {
        let pct = (task.completed_criteria as f64 / task.total_criteria as f64 * 100.0) as u32;
        let filled = (pct / 10) as usize;
        let empty = 10 - filled;
        let bar = format!("{}{}",  "█".repeat(filled), "░".repeat(empty));
        body.push_str(&format!("\n📊 **Progress:** {} {}/{} ({}%)\n", bar, task.completed_criteria, task.total_criteria, pct));
    }
    
    body.push_str("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n");
    
    // Specific call-to-action based on what we're actually doing
    let action_line = if let Some(ref goal) = task.goal {
        let goal_lower = goal.to_lowercase();
        if goal_lower.contains("postgres") || goal_lower.contains("database") {
            "Starting with database setup... 🗄️"
        } else if goal_lower.contains("redis") || goal_lower.contains("cache") {
            "Spinning up cache layer... ⚡"
        } else if goal_lower.contains("kafka") || goal_lower.contains("event") {
            "Wiring up event streams... 📡"
        } else if goal_lower.contains("kubernetes") || goal_lower.contains("k8s") || goal_lower.contains("infra") {
            "Deploying infrastructure... 🏗️"
        } else if goal_lower.contains("api") || goal_lower.contains("backend") {
            "Building the backend... 🔧"
        } else if goal_lower.contains("frontend") || goal_lower.contains("ui") {
            "Crafting the interface... 🎨"
        } else if goal_lower.contains("test") {
            "Running the test suite... 🧪"
        } else {
            "Getting to work... 💪"
        }
    } else {
        "Getting to work... 💪"
    };
    body.push_str(&format!("\n*{}*", action_line));
    
    // Use "response" type so it appears as a visible message in the agent dialog
    let content = serde_json::json!({
        "type": "response",
        "body": body
    });
    
    let variables = serde_json::json!({
        "input": {
            "agentSessionId": session_id,
            "content": content
        }
    });
    
    let response = client
        .post(LINEAR_API_URL)
        .header("Authorization", &state.linear_token)
        .header("Content-Type", "application/json")
        .json(&serde_json::json!({
            "query": query,
            "variables": variables
        }))
        .send()
        .await?;
    
    let status = response.status();
    if !status.is_success() {
        let body = response.text().await.unwrap_or_default();
        warn!("Failed to post init activity ({}): {}", status, body);
    } else {
        info!("📋 Posted init activity to Linear");
    }
    
    Ok(())
}

/// Post completion summary showing work accomplished, MCP tools and skills
async fn post_completion_summary(
    state: &AppState,
    session_id: &str,
    model: &str,
    tools: &[String],
    tools_used: &[String],
    skills: &[String],
    files_created: &[String],
    files_modified: &[String],
    subagents_spawned: &[String],
    duration_ms: Option<u64>,
    cost_usd: Option<f64>,
    num_turns: Option<u64>,
) -> Result<()> {
    let client = reqwest::Client::new();
    
    let query = r#"
        mutation AddAgentActivity($input: AgentActivityCreateInput!) {
            agentActivityCreate(input: $input) {
                success
            }
        }
    "#;
    
    // Build celebration-worthy completion summary
    let task = &state.task_context;
    
    // Agent personality
    let agent_name = task.agent.as_deref().unwrap_or("Agent");
    let agent_emoji = get_agent_emoji(agent_name);
    
    let mut body = format!("{} **{} completed the mission!**\n\n", agent_emoji, agent_name.to_uppercase());
    
    // Task completed
    if let Some(ref title) = task.title {
        body.push_str(&format!("📋 **{}**\n\n", title));
    }
    
    // Stats banner
    body.push_str("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n");
    let mut stats = Vec::new();
    if let Some(ms) = duration_ms {
        let secs = ms as f64 / 1000.0;
        if secs >= 60.0 {
            stats.push(format!("⏱️ {:.0}m {:.0}s", (secs / 60.0).floor(), secs % 60.0));
        } else {
            stats.push(format!("⏱️ {:.1}s", secs));
        }
    }
    if let Some(cost) = cost_usd {
        stats.push(format!("💰 ${:.4}", cost));
    }
    if let Some(turns) = num_turns {
        stats.push(format!("🔄 {} turns", turns));
    }
    if !stats.is_empty() {
        body.push_str(&format!("{}\n", stats.join(" │ ")));
    }
    body.push_str("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n\n");
    
    // What was built - the deliverables
    let total_files = files_created.len() + files_modified.len();
    if total_files > 0 {
        body.push_str("📦 **Deliverables:**\n");
        if !files_created.is_empty() {
            for file in files_created.iter().take(8) {
                let filename = file.rsplit('/').next().unwrap_or(file);
                body.push_str(&format!("  ✅ Created `{}`\n", filename));
            }
            if files_created.len() > 8 {
                body.push_str(&format!("  ... +{} more files\n", files_created.len() - 8));
            }
        }
        if !files_modified.is_empty() {
            for file in files_modified.iter().take(5) {
                let filename = file.rsplit('/').next().unwrap_or(file);
                body.push_str(&format!("  ✏️ Updated `{}`\n", filename));
            }
            if files_modified.len() > 5 {
                body.push_str(&format!("  ... +{} more modified\n", files_modified.len() - 5));
            }
        }
        body.push_str("\n");
    }
    
    // Subagents spawned
    if !subagents_spawned.is_empty() {
        body.push_str(&format!("🤖 **Delegated to:** {}\n\n", subagents_spawned.join(", ")));
    }
    
    // Tools used - only show the ones that were actually used
    let mcp_used: Vec<_> = tools_used.iter()
        .filter(|t| t.starts_with("mcp__"))
        .map(|t| t.strip_prefix("mcp__cto-tools__").unwrap_or(t).to_string())
        .collect();
    
    if !mcp_used.is_empty() {
        body.push_str(&format!("🔧 **Tools Used ({}):** {}\n", mcp_used.len(), mcp_used.join(", ")));
    }
    
    // Model
    body.push_str(&format!("🧠 **Model:** `{}`\n\n", model));
    
    // Acceptance criteria with progress bar
    if task.total_criteria > 0 {
        let pct = (task.completed_criteria as f64 / task.total_criteria as f64 * 100.0) as u32;
        let filled = (pct / 10) as usize;
        let empty = 10 - filled;
        let bar = format!("{}{}", "█".repeat(filled), "░".repeat(empty));
        let status = if pct >= 90 { "✅" } else if pct >= 50 { "🟡" } else { "🔴" };
        body.push_str(&format!("{} **Progress:** {} {}/{} ({}%)\n", status, bar, task.completed_criteria, task.total_criteria, pct));
    }
    
    // Specific sign-off based on what was accomplished
    let signoff = if !files_created.is_empty() {
        let count = files_created.len();
        if count == 1 {
            format!("Done! Created 1 file, ready for review. {}", agent_emoji)
        } else {
            format!("Done! {} files created, ready for review. {}", count, agent_emoji)
        }
    } else if !files_modified.is_empty() {
        format!("Done! Updated {} files. {}", files_modified.len(), agent_emoji)
    } else {
        format!("All done! {}", agent_emoji)
    };
    body.push_str(&format!("\n*{}*", signoff));
    
    let content = serde_json::json!({
        "type": "response",
        "body": body
    });
    
    let variables = serde_json::json!({
        "input": {
            "agentSessionId": session_id,
            "content": content
        }
    });
    
    let response = client
        .post(LINEAR_API_URL)
        .header("Authorization", &state.linear_token)
        .header("Content-Type", "application/json")
        .json(&serde_json::json!({
            "query": query,
            "variables": variables
        }))
        .send()
        .await?;
    
    let status = response.status();
    if !status.is_success() {
        let body = response.text().await.unwrap_or_default();
        warn!("Failed to post completion summary ({}): {}", status, body);
    } else {
        info!("📊 Posted completion summary to Linear");
    }
    
    Ok(())
}

async fn create_linear_session(state: &AppState, _model: &str, _tools: &[String], _skills: &[String]) -> Result<String> {
    let client = reqwest::Client::new();
    
    // First resolve the issue identifier to UUID
    let issue_id = resolve_issue_id(&state.linear_token, &state.issue_identifier).await?;
    
    let query = r#"
        mutation CreateAgentSession($input: AgentSessionCreateOnIssue!) {
            agentSessionCreateOnIssue(input: $input) {
                success
                agentSession {
                    id
                    status
                }
            }
        }
    "#;
    
    let variables = serde_json::json!({
        "input": {
            "issueId": issue_id
        }
    });
    
    let response = client
        .post(LINEAR_API_URL)
        .header("Authorization", &state.linear_token)
        .header("Content-Type", "application/json")
        .json(&serde_json::json!({
            "query": query,
            "variables": variables
        }))
        .send()
        .await
        .context("Failed to send Linear request")?;
    
    let body: serde_json::Value = response.json().await?;
    
    if let Some(errors) = body.get("errors") {
        error!("Linear API errors: {:?}", errors);
        anyhow::bail!("Linear API error: {:?}", errors);
    }
    
    let session_id = body["data"]["agentSessionCreateOnIssue"]["agentSession"]["id"]
        .as_str()
        .context("Missing session ID in response")?
        .to_string();
    
    info!("Created Linear session: {}", session_id);
    Ok(session_id)
}

/// Process a tool call and return narrative description if significant
/// Returns (should_post, narrative_text, new_phase)
fn process_tool_for_narrative(
    tool_name: &str, 
    params: &serde_json::Value,
    narrative: &NarrativeState
) -> (bool, Option<String>, Option<NarrativePhase>) {
    if let Some((new_phase, description, is_significant)) = narrate_tool_call(tool_name, params, narrative) {
        // Only post if significant
        if is_significant {
            (true, Some(description), Some(new_phase))
        } else {
            // Update phase but don't post
            (false, None, Some(new_phase))
        }
    } else {
        // Not recognized - skip
        (false, None, None)
    }
}

/// Add activity to Linear session
/// Linear activity types: action, error, thought, response, elicitation
/// Content structure varies by type:
///   - response: {"type": "response", "body": "..."}
///   - thought: {"type": "thought", "body": "..."}
///   - action: {"type": "action", "action": "ToolName", "parameter": "..."}
///   - error: {"type": "error", "body": "..."}
async fn add_linear_activity(state: &AppState, session_id: &str, entry: &LogEntry) -> Result<()> {
    let client = reqwest::Client::new();
    
    let query = r#"
        mutation AddAgentActivity($input: AgentActivityCreateInput!) {
            agentActivityCreate(input: $input) {
                success
            }
        }
    "#;
    
    // Get current narrative state for context
    let narrative_state = {
        let session = state.session.read().await;
        session.narrative.clone()
    };
    
    // Build activity content based on entry type
    // Handle both Claude CLI and Droid/Factory formats
    // Use narrative descriptions for tool calls when available
    let content: Option<serde_json::Value> = match entry.entry_type.as_deref() {
        // =========================================================================
        // Droid/Factory format: direct tool_call events
        // {"type":"tool_call","toolName":"LS","parameters":{"directory_path":"..."}}
        // =========================================================================
        Some("tool_call") => {
            let tool_name = entry.extra.get("toolName")
                .and_then(|v| v.as_str())
                .or_else(|| entry.extra.get("toolId").and_then(|v| v.as_str()))
                .or_else(|| entry.name.as_deref())
                .unwrap_or("unknown");
            let params = entry.extra.get("parameters")
                .cloned()
                .unwrap_or_else(|| serde_json::json!({}));
            
            // Try narrative description first
            let (should_post, narrative, new_phase) = process_tool_for_narrative(tool_name, &params, &narrative_state);
            
            // Update narrative phase if changed
            if let Some(phase) = new_phase {
                let mut session = state.session.write().await;
                if phase != session.narrative.phase {
                    info!("{} Phase: {:?} → {:?}", phase.emoji(), session.narrative.phase, phase);
                    session.narrative.phase = phase;
                }
                session.narrative.actions_in_phase += 1;
            }
            
            if should_post {
                if let Some(text) = narrative {
                    // Post as thought (narrative description)
                    Some(serde_json::json!({
                        "type": "thought",
                        "body": text
                    }))
                } else {
                    None
                }
            } else {
                // Not significant - skip
                None
            }
        }
        
        // =========================================================================
        // Droid/Factory format: tool_result events
        // {"type":"tool_result","toolId":"LS","value":"...","isError":false}
        // =========================================================================
        Some("tool_result") => {
            // Skip tool results - they're contextual, not activities
            // The Linear agent dialog shows action+response, not raw results
            None
        }
        
        // =========================================================================
        // Droid/Factory format: message events with role
        // {"type":"message","role":"assistant","text":"..."}
        // {"type":"message","role":"user","text":"..."}
        // =========================================================================
        Some("message") => {
            let role = entry.extra.get("role").and_then(|v| v.as_str());
            match role {
                Some("assistant") => {
                    let text = entry.extra.get("text")
                        .and_then(|v| v.as_str())
                        .unwrap_or("");
                    if !text.is_empty() {
                        Some(serde_json::json!({
                            "type": "response",
                            "body": text
                        }))
                    } else {
                        None
                    }
                }
                Some("user") => {
                    // Skip user messages - not needed in agent activity
                    None
                }
                _ => {
                    debug!("Skipping message with unknown role: {:?}", role);
                    None
                }
            }
        }
        
        // =========================================================================
        // Claude format: direct tool_use events (legacy/non-Claude CLIs)
        // =========================================================================
        Some("tool_use") | Some("tool") => {
            let tool_name = entry.tool_name.as_ref()
                .or(entry.name.as_ref())
                .map(|s| s.as_str())
                .unwrap_or("unknown");
            let input = entry.tool_input.as_ref()
                .or(entry.input.as_ref())
                .cloned()
                .unwrap_or_else(|| serde_json::json!({}));
            
            // Try narrative description
            let (should_post, narrative, new_phase) = process_tool_for_narrative(tool_name, &input, &narrative_state);
            
            if let Some(phase) = new_phase {
                let mut session = state.session.write().await;
                if phase != session.narrative.phase {
                    session.narrative.phase = phase;
                }
            }
            
            if should_post {
                if let Some(text) = narrative {
                    Some(serde_json::json!({
                        "type": "thought",
                        "body": text
                    }))
                } else {
                    None
                }
            } else {
                None
            }
        }
        
        // =========================================================================
        // Claude format: assistant messages (may contain text or tool_use)
        // {"type":"assistant","message":{"content":[{"type":"text","text":"..."}]}}
        // {"type":"assistant","message":{"content":[{"type":"tool_use","name":"..."}]}}
        // =========================================================================
        Some("assistant") => {
            // Check for tool_use in Claude's nested format
            if let Some((tool_name, tool_input)) = extract_claude_tool_use(entry) {
                // Try narrative description
                let (should_post, narrative, new_phase) = process_tool_for_narrative(&tool_name, &tool_input, &narrative_state);
                
                if let Some(phase) = new_phase {
                    let mut session = state.session.write().await;
                    if phase != session.narrative.phase {
                        info!("{} Phase: {:?} → {:?}", phase.emoji(), session.narrative.phase, phase);
                        session.narrative.phase = phase;
                    }
                    session.narrative.actions_in_phase += 1;
                }
                
                if should_post {
                    if let Some(text) = narrative {
                        Some(serde_json::json!({
                            "type": "thought",
                            "body": text
                        }))
                    } else {
                        None
                    }
                } else {
                    // Not significant - skip
                    None
                }
            } else if let Some(text) = extract_claude_text(entry) {
                Some(serde_json::json!({
                    "type": "response",
                    "body": text
                }))
            } else {
                None // Skip empty assistant messages
            }
        }
        
        // =========================================================================
        // Other event types
        // =========================================================================
        Some("text") => {
            let msg = extract_claude_text(entry)
                .or_else(|| entry.content.clone())
                .unwrap_or_default();
            if msg.is_empty() {
                None
            } else {
                Some(serde_json::json!({
                    "type": "response",
                    "body": msg
                }))
            }
        }
        Some("user") => {
            // User events contain tool results - skip
            None
        }
        Some("system") => {
            // System events (like init) are handled elsewhere
            None
        }
        Some("error") => {
            let msg = extract_claude_text(entry)
                .or_else(|| entry.content.clone())
                .or_else(|| entry.extra.get("message").and_then(|v| v.as_str()).map(|s| s.to_string()))
                .unwrap_or_else(|| "Unknown error".to_string());
            Some(serde_json::json!({
                "type": "error",
                "body": msg
            }))
        }
        Some("result") => {
            // Final result event
            Some(serde_json::json!({
                "type": "thought",
                "body": "✅ Session completed"
            }))
        }
        
        // =========================================================================
        // Codex format events (for future support)
        // =========================================================================
        Some("thread.started") | Some("turn.started") | Some("turn.completed") => {
            // Codex lifecycle events - skip
            None
        }
        Some("turn.failed") => {
            let msg = entry.extra.get("error")
                .and_then(|e| e.get("message"))
                .and_then(|m| m.as_str())
                .unwrap_or("Turn failed");
            Some(serde_json::json!({
                "type": "error",
                "body": msg
            }))
        }
        
        _ => {
            // Skip unknown types
            debug!("Skipping unknown event type: {:?}", entry.entry_type);
            None
        }
    };
    
    // Skip if no content to post
    let content = match content {
        Some(c) => c,
        None => return Ok(()),
    };
    
    let variables = serde_json::json!({
        "input": {
            "agentSessionId": session_id,
            "content": content
        }
    });
    
    let response = client
        .post(LINEAR_API_URL)
        .header("Authorization", &state.linear_token)
        .header("Content-Type", "application/json")
        .json(&serde_json::json!({
            "query": query,
            "variables": variables
        }))
        .send()
        .await?;
    
    let status = response.status();
    if !status.is_success() {
        let body = response.text().await.unwrap_or_default();
        warn!("Failed to add activity ({}): {}", status, body);
    }
    
    Ok(())
}

// =============================================================================
// HTTP Handlers
// =============================================================================

/// Health check endpoint
async fn health() -> &'static str {
    "OK"
}

/// Ingest endpoint - receives logs from FluentD
async fn ingest(
    State(state): State<Arc<AppState>>,
    Json(entry): Json<LogEntry>,
) -> StatusCode {
    debug!("Received log entry: {:?}", entry);
    
    let mut session = state.session.write().await;
    session.total_entries += 1;
    
    // Handle init message - create Linear session
    // Claude format: {"type":"system","subtype":"init",...}
    // Generic format: {"type":"init",...}
    let is_init = is_claude_init(&entry) || entry.entry_type.as_deref() == Some("init");
    
    if is_init {
        let model = entry.model.clone().unwrap_or_else(|| "unknown".to_string());
        let tools = entry.tools.clone().unwrap_or_default();
        let skills = entry.skills.clone().unwrap_or_default();
        
        session.model = Some(model.clone());
        session.tools = tools.clone();
        session.skills = skills.clone();
        
        info!("🚀 Init event received: model={}, tools={}, skills={}", 
              model, tools.len(), skills.len());
        
        drop(session); // Release lock before async call
        
        match create_linear_session(&state, &model, &tools, &skills).await {
            Ok(session_id) => {
                // Post init activity with model/tools/skills summary
                if let Err(e) = post_init_activity(&state, &session_id, &model, &tools, &skills).await {
                    warn!("Failed to post init activity: {}", e);
                }
                
                // Post milestone comment on the issue itself
                let goal = state.task_context.goal.as_deref().unwrap_or("");
                let goal_preview = if goal.len() > 200 { format!("{}...", &goal[..200]) } else { goal.to_string() };
                if let Err(e) = post_milestone_comment(&state, MilestoneType::TaskStarted, &goal_preview).await {
                    warn!("Failed to post task started milestone: {}", e);
                }
                
                let mut session = state.session.write().await;
                session.session_id = Some(session_id.clone());
                session.start_time = Some(std::time::Instant::now());
                info!("✅ Linear session created: {}", session_id);
            }
            Err(e) => {
                error!("❌ Failed to create Linear session: {}", e);
                return StatusCode::INTERNAL_SERVER_ERROR;
            }
        }
        
        return StatusCode::OK;
    }
    
    // Track tool usage and file operations - handles both Claude and Droid formats
    match entry.entry_type.as_deref() {
        // Droid format: {"type":"tool_call","toolName":"LS",...}
        Some("tool_call") => {
            let tool_name = entry.extra.get("toolName")
                .and_then(|v| v.as_str())
                .or_else(|| entry.extra.get("toolId").and_then(|v| v.as_str()))
                .or_else(|| entry.name.as_deref());
            if let Some(name) = tool_name {
                if !session.tools_used.contains(&name.to_string()) {
                    session.tools_used.push(name.to_string());
                    debug!("Tool used (Droid): {}", name);
                }
                
                // Track file operations
                let params = entry.extra.get("parameters");
                match name {
                    "Write" | "write" => {
                        if let Some(file) = params.and_then(|p| p.get("file_path").or(p.get("path"))).and_then(|v| v.as_str()) {
                            if !session.files_created.contains(&file.to_string()) {
                                session.files_created.push(file.to_string());
                                debug!("File created: {}", file);
                            }
                        }
                    }
                    "Edit" | "edit" => {
                        if let Some(file) = params.and_then(|p| p.get("file_path").or(p.get("path"))).and_then(|v| v.as_str()) {
                            if !session.files_modified.contains(&file.to_string()) {
                                session.files_modified.push(file.to_string());
                                debug!("File modified: {}", file);
                            }
                        }
                    }
                    "Task" | "task" => {
                        if let Some(desc) = params.and_then(|p| p.get("description")).and_then(|v| v.as_str()) {
                            let short_desc = if desc.len() > 50 { format!("{}...", &desc[..50]) } else { desc.to_string() };
                            session.subagents_spawned.push(short_desc);
                            debug!("Subagent spawned");
                        }
                    }
                    _ => {}
                }
            }
        }
        // Legacy format: {"type":"tool_use"} or {"type":"tool"}
        Some("tool_use") | Some("tool") => {
            if let Some(tool_name) = entry.tool_name.as_ref().or(entry.name.as_ref()) {
                if !session.tools_used.contains(tool_name) {
                    session.tools_used.push(tool_name.clone());
                    debug!("Tool used: {}", tool_name);
                }
            }
        }
        // Claude format: {"type":"assistant","message":{"content":[{"type":"tool_use"}]}}
        Some("assistant") => {
            if let Some((tool_name, tool_input)) = extract_claude_tool_use(&entry) {
                if !session.tools_used.contains(&tool_name) {
                    session.tools_used.push(tool_name.clone());
                    debug!("Tool used (Claude): {}", tool_name);
                }
                
                // Track file operations from Claude tool use
                match tool_name.as_str() {
                    "Write" | "write" => {
                        if let Some(file) = tool_input.get("file_path").or(tool_input.get("path")).and_then(|v| v.as_str()) {
                            if !session.files_created.contains(&file.to_string()) {
                                session.files_created.push(file.to_string());
                                debug!("File created (Claude): {}", file);
                            }
                        }
                    }
                    "Edit" | "edit" => {
                        if let Some(file) = tool_input.get("file_path").or(tool_input.get("path")).and_then(|v| v.as_str()) {
                            if !session.files_modified.contains(&file.to_string()) {
                                session.files_modified.push(file.to_string());
                                debug!("File modified (Claude): {}", file);
                            }
                        }
                    }
                    "Task" | "task" => {
                        if let Some(desc) = tool_input.get("description").and_then(|v| v.as_str()) {
                            let short_desc = if desc.len() > 50 { format!("{}...", &desc[..50]) } else { desc.to_string() };
                            session.subagents_spawned.push(short_desc);
                            debug!("Subagent spawned (Claude)");
                        }
                    }
                    _ => {}
                }
            }
        }
        _ => {}
    }
    
    // Track errors
    if entry.entry_type.as_deref() == Some("error") {
        session.errors += 1;
    }
    
    // Handle result event specially - post completion summary
    if entry.entry_type.as_deref() == Some("result") {
        if let Some(ref session_id) = session.session_id {
            let session_id = session_id.clone();
            let model = session.model.clone().unwrap_or_else(|| "unknown".to_string());
            let tools = session.tools.clone();
            let tools_used = session.tools_used.clone();
            let skills = session.skills.clone();
            let files_created = session.files_created.clone();
            let files_modified = session.files_modified.clone();
            let subagents_spawned = session.subagents_spawned.clone();
            
            // Extract stats from result event
            let duration_ms = entry.extra.get("duration_ms")
                .and_then(|v| v.as_u64());
            let cost_usd = entry.extra.get("total_cost_usd")
                .and_then(|v| v.as_f64());
            let num_turns = entry.extra.get("num_turns")
                .and_then(|v| v.as_u64());
            
            drop(session); // Release lock before async call
            
            if let Err(e) = post_completion_summary(
                &state, &session_id, &model, &tools, &tools_used, &skills,
                &files_created, &files_modified, &subagents_spawned,
                duration_ms, cost_usd, num_turns
            ).await {
                warn!("Failed to post completion summary: {}", e);
            }
            
            // Post task completed milestone to the issue itself
            let duration_secs = duration_ms.map(|ms| ms as f64 / 1000.0).unwrap_or(0.0);
            let cost = cost_usd.unwrap_or(0.0);
            let deliverables = if !files_created.is_empty() {
                format!("Created {} files: {}", files_created.len(), 
                    files_created.iter().take(3).map(|f| f.rsplit('/').next().unwrap_or(f)).collect::<Vec<_>>().join(", "))
            } else {
                String::new()
            };
            if let Err(e) = post_milestone_comment(
                &state, 
                MilestoneType::TaskCompleted { duration_secs, cost_usd: cost },
                &deliverables
            ).await {
                warn!("Failed to post task completed milestone: {}", e);
            }
            
            return StatusCode::OK;
        }
    }
    
    // Add activity to Linear if session exists
    if let Some(ref session_id) = session.session_id {
        let session_id = session_id.clone();
        drop(session); // Release lock before async call
        
        if let Err(e) = add_linear_activity(&state, &session_id, &entry).await {
            warn!("Failed to add Linear activity: {}", e);
        }
    }
    
    StatusCode::OK
}

/// Batch ingest endpoint - receives array of logs from FluentD
async fn ingest_batch(
    State(state): State<Arc<AppState>>,
    Json(entries): Json<Vec<LogEntry>>,
) -> StatusCode {
    for entry in entries {
        let _ = ingest(State(state.clone()), Json(entry)).await;
    }
    StatusCode::OK
}

/// Status endpoint - returns current session state
async fn status(State(state): State<Arc<AppState>>) -> Json<serde_json::Value> {
    let session = state.session.read().await;
    Json(serde_json::json!({
        "session_id": session.session_id,
        "model": session.model,
        "tools_count": session.tools.len(),
        "tools_used": session.tools_used.len(),
        "skills_count": session.skills.len(),
        "total_entries": session.total_entries,
        "errors": session.errors
    }))
}

// =============================================================================
// File Watcher (fallback mode)
// =============================================================================

async fn watch_file(state: Arc<AppState>, path: PathBuf) -> Result<()> {
    use tokio::fs::File;
    use tokio::io::{AsyncBufReadExt, BufReader};
    
    info!("Watching file: {:?}", path);
    
    // Wait for file to exist
    while !path.exists() {
        tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;
    }
    
    let file = File::open(&path).await?;
    let reader = BufReader::new(file);
    let mut lines = reader.lines();
    
    // Continuously watch for new content (tail -f style)
    loop {
        match lines.next_line().await? {
            Some(line) => {
                if line.trim().is_empty() {
                    continue;
                }
                
                match serde_json::from_str::<LogEntry>(&line) {
                    Ok(entry) => {
                        let _ = ingest(State(state.clone()), Json(entry)).await;
                    }
                    Err(e) => {
                        debug!("Failed to parse line: {} - {}", e, line);
                    }
                }
            }
            None => {
                // No new data - wait and check again (poll for new content)
                tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
            }
        }
    }
}

// =============================================================================
// Main
// =============================================================================

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize tracing
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::from_default_env()
                .add_directive("linear_sidecar=info".parse()?)
        )
        .init();
    
    info!("Linear sidecar starting...");
    
    // Load configuration from environment
    let linear_token = env::var("LINEAR_OAUTH_TOKEN")
        .context("LINEAR_OAUTH_TOKEN not set")?;
    let issue_identifier = env::var("LINEAR_ISSUE_IDENTIFIER")
        .context("LINEAR_ISSUE_IDENTIFIER not set")?;
    let agent_name = env::var("CTO_AGENT_NAME")
        .unwrap_or_else(|_| "unknown".to_string());
    let log_source = env::var("LOG_SOURCE")
        .unwrap_or_else(|_| "file".to_string());
    
    // Load workspace path and task context
    let workspace_path = env::var("WORKSPACE_PATH")
        .map(PathBuf::from)
        .ok();
    
    // Try to load task context from prompt.md
    let mut task_context = if let Some(ref ws_path) = workspace_path {
        let prompt_path = env::var("TASK_PROMPT_PATH")
            .map(PathBuf::from)
            .unwrap_or_else(|_| ws_path.join("prompt.md"));
        
        if prompt_path.exists() {
            match std::fs::read_to_string(&prompt_path) {
                Ok(content) => {
                    let ctx = parse_task_context(&content);
                    info!("📋 Loaded task context: {:?}", ctx.title);
                    ctx
                }
                Err(e) => {
                    warn!("Failed to read prompt.md: {}", e);
                    TaskContext::default()
                }
            }
        } else {
            info!("No prompt.md found at {:?}", prompt_path);
            TaskContext::default()
        }
    } else {
        TaskContext::default()
    };
    
    // Try to load acceptance criteria
    if let Some(ref ws_path) = workspace_path {
        let acceptance_path = env::var("TASK_ACCEPTANCE_PATH")
            .map(PathBuf::from)
            .unwrap_or_else(|_| ws_path.join("acceptance-criteria.md"));
        
        if acceptance_path.exists() {
            if let Ok(content) = std::fs::read_to_string(&acceptance_path) {
                let (total, completed) = parse_acceptance_criteria(&content);
                task_context.total_criteria = total;
                task_context.completed_criteria = completed;
                info!("📊 Loaded acceptance criteria: {}/{} complete", completed, total);
            }
        }
    }
    
    info!("Issue: {}, Agent: {}, Source: {}", issue_identifier, agent_name, log_source);
    
    // Create shared state
    let state = Arc::new(AppState {
        session: RwLock::new(SessionState::default()),
        linear_token,
        issue_identifier,
        agent_name,
        task_context,
        workspace_path,
    });
    
    match log_source.as_str() {
        "http" => {
            // HTTP ingest mode - receive from FluentD
            let port: u16 = env::var("HTTP_PORT")
                .unwrap_or_else(|_| "8080".to_string())
                .parse()
                .unwrap_or(8080);
            
            let app = Router::new()
                .route("/health", get(health))
                .route("/ingest", post(ingest))
                .route("/ingest/batch", post(ingest_batch))
                .route("/status", get(status))
                .with_state(state);
            
            let addr = format!("0.0.0.0:{}", port);
            info!("Starting HTTP server on {}", addr);
            
            let listener = tokio::net::TcpListener::bind(&addr).await?;
            axum::serve(listener, app).await?;
        }
        "file" | _ => {
            // File watching mode - read JSONL stream
            let stream_file = env::var("STREAM_FILE")
                .unwrap_or_else(|_| "/workspace/stream.jsonl".to_string());
            
            watch_file(state, PathBuf::from(stream_file)).await?;
        }
    }
    
    Ok(())
}
