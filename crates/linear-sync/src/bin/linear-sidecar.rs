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
}

/// Shared application state
struct AppState {
    session: RwLock<SessionState>,
    linear_token: String,
    issue_identifier: String,
    agent_name: String,
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

/// Create agent session on Linear issue
/// Note: Linear's AgentSessionCreateOnIssue now only accepts issueId
/// Model, tools, skills are added via activities
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
    
    // Build activity content based on entry type
    // Handle Claude CLI's nested message format
    let content: Option<serde_json::Value> = match entry.entry_type.as_deref() {
        // Direct tool_use events (non-Claude)
        Some("tool_use") | Some("tool") => {
            let tool_name = entry.tool_name.as_ref()
                .or(entry.name.as_ref())
                .map(|s| s.as_str())
                .unwrap_or("unknown");
            let input = entry.tool_input.as_ref()
                .or(entry.input.as_ref())
                .map(|v| serde_json::to_string(v).unwrap_or_default())
                .unwrap_or_default();
            Some(serde_json::json!({
                "type": "action",
                "action": tool_name,
                "parameter": input
            }))
        }
        // Claude assistant messages (may contain text or tool_use)
        Some("assistant") => {
            // Check for tool_use in Claude's nested format
            if let Some((tool_name, tool_input)) = extract_claude_tool_use(entry) {
                let input_str = serde_json::to_string(&tool_input).unwrap_or_default();
                Some(serde_json::json!({
                    "type": "action",
                    "action": tool_name,
                    "parameter": input_str
                }))
            } else if let Some(text) = extract_claude_text(entry) {
                Some(serde_json::json!({
                    "type": "response",
                    "body": text
                }))
            } else {
                None // Skip empty assistant messages
            }
        }
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
                let mut session = state.session.write().await;
                session.session_id = Some(session_id.clone());
                info!("✅ Linear session created: {}", session_id);
            }
            Err(e) => {
                error!("❌ Failed to create Linear session: {}", e);
                return StatusCode::INTERNAL_SERVER_ERROR;
            }
        }
        
        return StatusCode::OK;
    }
    
    // Track tool usage - Claude embeds tool_use in assistant messages
    if entry.entry_type.as_deref() == Some("tool_use") || entry.entry_type.as_deref() == Some("tool") {
        if let Some(tool_name) = entry.tool_name.as_ref().or(entry.name.as_ref()) {
            if !session.tools_used.contains(tool_name) {
                session.tools_used.push(tool_name.clone());
                debug!("Tool used: {}", tool_name);
            }
        }
    } else if entry.entry_type.as_deref() == Some("assistant") {
        // Check for tool_use in Claude's nested format
        if let Some((tool_name, _)) = extract_claude_tool_use(&entry) {
            if !session.tools_used.contains(&tool_name) {
                session.tools_used.push(tool_name.clone());
                debug!("Tool used (Claude): {}", tool_name);
            }
        }
    }
    
    // Track errors
    if entry.entry_type.as_deref() == Some("error") {
        session.errors += 1;
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
    
    info!("Issue: {}, Agent: {}, Source: {}", issue_identifier, agent_name, log_source);
    
    // Create shared state
    let state = Arc::new(AppState {
        session: RwLock::new(SessionState::default()),
        linear_token,
        issue_identifier,
        agent_name,
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
