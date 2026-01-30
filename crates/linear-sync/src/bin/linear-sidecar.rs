//! Linear Sidecar Binary
//!
//! This binary runs alongside agent pods to sync status and stream
//! activities to Linear's agent dialog.
//!
//! # Usage
//!
//! ```bash
//! # Required environment variables:
//! LINEAR_OAUTH_TOKEN=lin_oauth_...  # Linear OAuth token
//! LINEAR_ISSUE_IDENTIFIER=CTOPA-123 # Or LINEAR_SESSION_ID if session already exists
//! CLI_TYPE=claude                    # Optional: auto-detected if not set
//! STREAM_FILE=/workspace/stream.jsonl # Optional: defaults to /workspace/stream.jsonl
//! ```
//!
//! The sidecar:
//! 1. Creates a session on the Linear issue (if `LINEAR_SESSION_ID` not set)
//! 2. Tails the stream file for CLI output
//! 3. Parses each line using the appropriate parser
//! 4. Emits activities to the Linear agent dialog
//! 5. Emits a completion summary when the stream ends

use anyhow::{Context, Result};
use linear_sink::{
    parsers::ParserRegistry, AgentActivityEmitter, InitInfo, LinearAgentEmitter, LinearClient,
    ParsedActivity, StreamParser, StreamStats,
};
use std::collections::HashSet;
use std::path::Path;
use std::time::Duration;
use tokio::fs::File;
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::time::sleep;
use tracing::{debug, info, warn};

// =============================================================================
// MCP Tool Detection
// =============================================================================
// 
// Native Claude tools (IGNORE these):
//   Read, Write, Edit, Bash, Glob, Grep, LS, WebSearch, WebFetch, 
//   MultiEdit, TodoRead, TodoWrite, Task, AskFollowupQuestion, etc.
//
// MCP Tools (TRACK these) - from CTO tools server:
//   context7_*, octocode_*, firecrawl_*, openmemory_*, github_*
//   Also any tool called via mcp__cto-tools__*
// =============================================================================

/// Known MCP tool prefixes from the CTO tools server
const MCP_TOOL_PREFIXES: &[&str] = &[
    "context7_",
    "octocode_",
    "firecrawl_",
    "openmemory_",
    "github_",
    "mcp__",  // Generic MCP tool call prefix
];

/// Check if a tool name is an MCP tool (not a native Claude tool)
fn is_mcp_tool(name: &str) -> bool {
    MCP_TOOL_PREFIXES.iter().any(|prefix| name.starts_with(prefix))
}

/// Extract the base MCP tool name from a full tool call name
/// e.g., "mcp__cto-tools__context7_resolve_library_id" -> "context7_resolve_library_id"
fn normalize_mcp_tool_name(name: &str) -> String {
    if let Some(suffix) = name.strip_prefix("mcp__cto-tools__") {
        suffix.to_string()
    } else if let Some(suffix) = name.strip_prefix("mcp__cto__") {
        suffix.to_string()
    } else {
        name.to_string()
    }
}

/// Stream processing result
struct StreamResult {
    /// Stats from parsing
    stats: StreamStats,
    /// MCP tools available (from init)
    available_mcp_tools: Vec<String>,
    /// MCP tools actually used during session
    used_mcp_tools: HashSet<String>,
    /// Skills available (from init)
    available_skills: Vec<String>,
    /// Skills referenced/used during session (by name in tool calls or responses)
    used_skills: HashSet<String>,
    /// Model name
    model: Option<String>,
}

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize tracing
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::from_default_env()
                .add_directive("linear_sidecar=info".parse().unwrap())
                .add_directive("linear_sink=info".parse().unwrap()),
        )
        .init();

    info!("=== Linear Sidecar Starting ===");

    // Get configuration from environment
    let token = std::env::var("LINEAR_OAUTH_TOKEN")
        .or_else(|_| std::env::var("LINEAR_API_TOKEN"))
        .context("LINEAR_OAUTH_TOKEN or LINEAR_API_TOKEN required")?;

    let stream_file = std::env::var("STREAM_FILE").unwrap_or_else(|_| "/workspace/stream.jsonl".to_string());

    info!("Stream file: {}", stream_file);

    // Create Linear client
    let client = LinearClient::new(&token)?;

    // Get or create session
    let session_id = get_or_create_session(&client).await?;
    info!("Session ID: {}", session_id);

    // Create emitter
    let emitter = LinearAgentEmitter::new(client, &session_id);

    // Emit initial thought
    emitter
        .emit_thought("🚀 Agent sidecar connected, waiting for CLI output...", true)
        .await?;

    // Wait for stream file to exist
    wait_for_file(&stream_file).await?;

    // Process the stream
    let result = process_stream_file(&stream_file, &emitter).await?;

    // Emit completion summary
    emit_completion_summary(&emitter, &result).await?;

    info!("=== Linear Sidecar Complete ===");
    Ok(())
}

/// Get existing session ID or create a new one
async fn get_or_create_session(client: &LinearClient) -> Result<String> {
    // First check if session ID is provided directly
    if let Ok(session_id) = std::env::var("LINEAR_SESSION_ID") {
        if !session_id.is_empty() {
            info!("Using existing session ID from environment");
            return Ok(session_id);
        }
    }

    // Otherwise, create a session on the issue
    let identifier = std::env::var("LINEAR_ISSUE_IDENTIFIER")
        .context("LINEAR_ISSUE_IDENTIFIER required (e.g., CTOPA-123)")?;

    info!("Creating session on issue: {}", identifier);
    client.create_session_on_issue_by_identifier(&identifier).await
}

/// Wait for the stream file to exist
async fn wait_for_file(path: &str) -> Result<()> {
    let path = Path::new(path);
    let max_wait = Duration::from_secs(120);
    let poll_interval = Duration::from_secs(1);
    let mut waited = Duration::ZERO;

    while !path.exists() {
        if waited >= max_wait {
            return Err(anyhow::anyhow!(
                "Timed out waiting for stream file: {}",
                path.display()
            ));
        }
        debug!("Waiting for stream file: {}", path.display());
        sleep(poll_interval).await;
        waited += poll_interval;
    }

    info!("Stream file found: {}", path.display());
    Ok(())
}

/// Process the stream file and emit activities
async fn process_stream_file(path: &str, emitter: &LinearAgentEmitter) -> Result<StreamResult> {
    let file = File::open(path).await.context("Failed to open stream file")?;
    let reader = BufReader::new(file);
    let mut lines = reader.lines();

    // Initialize parser registry
    let registry = ParserRegistry::new();
    let mut parser: Option<Box<dyn StreamParser>> = registry.from_env_or_detect(None);

    // Track state
    let mut available_mcp_tools: Vec<String> = Vec::new();
    let mut used_mcp_tools: HashSet<String> = HashSet::new();
    let mut available_skills: Vec<String> = Vec::new();
    let used_skills: HashSet<String> = HashSet::new(); // TODO: Skill usage detection is complex - leave empty for now
    let mut model: Option<String> = None;
    let mut init_emitted = false;
    let mut line_count = 0;

    // Process each line
    while let Some(line) = lines.next_line().await? {
        line_count += 1;

        if line.trim().is_empty() {
            continue;
        }

        // Auto-detect parser on first non-empty line if not set
        if parser.is_none() {
            parser = registry.detect_parser(&line);
            if parser.is_some() {
                let parser_id = parser.as_ref().map_or("unknown", |p| p.id());
                info!("Auto-detected parser: {}", parser_id);
            }
        }

        let Some(ref mut p) = parser else {
            warn!("No parser available, skipping line");
            continue;
        };
        // Rebind p to avoid type inference issues
        let p: &mut Box<dyn StreamParser> = p;

        // Try to extract init info from first few lines
        if !init_emitted && line_count <= 5 {
            if let Some(init_info) = extract_init_info(&line) {
                model = init_info.model.clone();
                
                // Extract MCP tools (filter out native tools by checking for known patterns)
                // MCP tools typically have prefixes like mcp__, context7_, octocode_, firecrawl_, etc.
                for tool in &init_info.tool_names {
                    if is_mcp_tool(tool) {
                        available_mcp_tools.push(tool.clone());
                    }
                }
                
                // Extract skills from mcp_servers (stored with "skill:" prefix)
                for server in &init_info.mcp_servers {
                    if let Some(skill_name) = server.strip_prefix("skill:") {
                        available_skills.push(skill_name.to_string());
                    }
                }
                
                // Emit initialization activity
                emit_init_activity(emitter, &init_info).await?;
                init_emitted = true;
            }
        }

        // Parse the line
        let result = p.parse_line(&line);

        // Emit activities
        for activity in result.activities {
            emit_activity(emitter, &activity).await?;

            // Track MCP tool usage (ignore native Claude tools)
            if let ParsedActivity::Action { ref name, .. } = activity {
                if is_mcp_tool(name) {
                    // Normalize the tool name for matching with available tools
                    let normalized = normalize_mcp_tool_name(name);
                    used_mcp_tools.insert(normalized);
                }
            }
        }
    }

    // Get final stats
    let stats = match parser {
        Some(p) => p.get_stats(),
        None => StreamStats::default(),
    };

    Ok(StreamResult {
        stats,
        available_mcp_tools,
        used_mcp_tools,
        available_skills,
        used_skills,
        model,
    })
}

/// Extract initialization info from a stream line (Claude-specific for now)
fn extract_init_info(line: &str) -> Option<InitInfo> {
    // Try to parse as JSON
    let value: serde_json::Value = serde_json::from_str(line).ok()?;

    // Claude's init message has type "init" or "system"
    let msg_type = value.get("type")?.as_str()?;
    
    if msg_type == "init" || msg_type == "system" {
        let mut info = InitInfo::new();

        // Extract model
        if let Some(model) = value.get("model").and_then(|v| v.as_str()) {
            info.model = Some(model.to_string());
        }

        // Extract tools - handle both string arrays and object arrays
        if let Some(tools) = value.get("tools").and_then(|v| v.as_array()) {
            info.tool_count = tools.len();
            info.tool_names = tools
                .iter()
                .filter_map(|t| {
                    // Handle string tools (Claude format: ["Read", "Write", "mcp__..."])
                    t.as_str()
                        .map(String::from)
                        // Handle object tools (other formats: [{"name": "tool"}])
                        .or_else(|| t.get("name").and_then(|n| n.as_str()).map(String::from))
                })
                .collect();
        }

        // Extract MCP servers
        if let Some(mcp) = value.get("mcp_servers").and_then(|v| v.as_array()) {
            info.mcp_servers = mcp
                .iter()
                .filter_map(|s| s.get("name").and_then(|n| n.as_str()))
                .map(String::from)
                .collect();
        }

        // Extract skills if present - skills can be strings or objects
        if let Some(agent_skills) = value.get("skills").and_then(|v| v.as_array()) {
            let skill_names: Vec<String> = agent_skills
                .iter()
                .filter_map(|s| {
                    // Handle both string skills and object skills with name field
                    s.as_str()
                        .map(String::from)
                        .or_else(|| s.get("name").and_then(|n| n.as_str()).map(String::from))
                })
                .collect();
            
            // Store in mcp_servers with "skill:" prefix for display
            if !skill_names.is_empty() {
                info.mcp_servers.extend(skill_names.iter().map(|s| format!("skill:{s}")));
            }
        }

        return Some(info);
    }

    None
}

/// Emit initialization activity with tools/skills summary
async fn emit_init_activity(emitter: &LinearAgentEmitter, init: &InitInfo) -> Result<()> {
    let mut sections = Vec::new();

    // Model section
    if let Some(ref model) = init.model {
        sections.push(format!("**Model:** {model}"));
    }

    // MCP Tools section
    if init.tool_count > 0 {
        let tool_preview: Vec<_> = init.tool_names.iter().take(10).cloned().collect();
        let tools_str = if tool_preview.len() < init.tool_count {
            format!("{} (+{} more)", tool_preview.join(", "), init.tool_count - tool_preview.len())
        } else {
            tool_preview.join(", ")
        };
        sections.push(format!("**MCP Tools ({}):** {}", init.tool_count, tools_str));
    }

    // MCP Servers section
    let servers: Vec<_> = init.mcp_servers.iter()
        .filter(|s| !s.starts_with("skill:"))
        .cloned()
        .collect();
    if !servers.is_empty() {
        sections.push(format!("**MCP Servers:** {}", servers.join(", ")));
    }

    // Skills section
    let skills: Vec<_> = init.mcp_servers.iter()
        .filter(|s| s.starts_with("skill:"))
        .map(|s| s.strip_prefix("skill:").unwrap_or(s))
        .collect();
    if !skills.is_empty() {
        sections.push(format!("**Skills ({}):** {}", skills.len(), skills.join(", ")));
    }

    let body = if sections.is_empty() {
        "🚀 Agent initialized".to_string()
    } else {
        format!("🚀 **Agent Initialized**\n\n{}", sections.join("\n"))
    };

    emitter.emit_thought(&body, false).await?;
    Ok(())
}

/// Emit a parsed activity to Linear
async fn emit_activity(emitter: &LinearAgentEmitter, activity: &ParsedActivity) -> Result<()> {
    match activity {
        ParsedActivity::Thought { body, ephemeral } => {
            emitter.emit_thought(body, *ephemeral).await?;
        }
        ParsedActivity::Action { name, input, result } => {
            if let Some(res) = result {
                emitter.emit_action_complete(name, input, res).await?;
            } else {
                emitter.emit_action(name, input).await?;
            }
        }
        ParsedActivity::Response { body } => {
            emitter.emit_response(body).await?;
        }
        ParsedActivity::Error { body } => {
            emitter.emit_error(body).await?;
        }
    }
    Ok(())
}

/// Emit completion summary with stats, tools used, and skills
async fn emit_completion_summary(emitter: &LinearAgentEmitter, result: &StreamResult) -> Result<()> {
    let mut sections = Vec::new();

    // Stats summary
    let stats_summary = result.stats.to_summary();
    if stats_summary != "Completed" {
        sections.push(format!("**Stats:** {stats_summary}"));
    }

    // Model
    if let Some(ref model) = result.model {
        sections.push(format!("**Model:** {model}"));
    }

    // MCP Tools - show available vs used with visual indicators
    if !result.available_mcp_tools.is_empty() {
        let used_count = result.used_mcp_tools.len();
        let total_count = result.available_mcp_tools.len();
        
        // Build list with used/unused indicators
        let tools_with_status: Vec<String> = result
            .available_mcp_tools
            .iter()
            .map(|tool| {
                if result.used_mcp_tools.contains(tool) {
                    format!("✅ {tool}")
                } else {
                    format!("⬜ {tool}")
                }
            })
            .collect();
        
        sections.push(format!(
            "**MCP Tools ({}/{} used):**\n{}",
            used_count,
            total_count,
            tools_with_status.join("\n")
        ));
    }

    // Skills - show available vs used with visual indicators
    if !result.available_skills.is_empty() {
        let used_count = result.used_skills.len();
        let total_count = result.available_skills.len();
        
        // Build list with used/unused indicators
        let skills_with_status: Vec<String> = result
            .available_skills
            .iter()
            .map(|skill| {
                if result.used_skills.contains(skill) {
                    format!("✅ {skill}")
                } else {
                    format!("⬜ {skill}")
                }
            })
            .collect();
        
        sections.push(format!(
            "**Skills ({}/{} used):**\n{}",
            used_count,
            total_count,
            skills_with_status.join("\n")
        ));
    }

    let body = if sections.is_empty() {
        "✅ **Claude Session Complete**".to_string()
    } else {
        format!("✅ **Claude Session Complete**\n\n{}", sections.join("\n\n"))
    };

    emitter.emit_response(&body).await?;
    Ok(())
}
