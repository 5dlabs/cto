use crate::db::Database;
use crate::error::AppError;
use chrono::Utc;
use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Manager, State};

const STUDIO_STATE_KEY: &str = "studio_state_v1";

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProjectRecord {
    pub id: String,
    pub name: String,
    pub summary: String,
    pub repository: Option<String>,
    pub prd_title: String,
    pub prd_content: String,
    pub workflow_summary: String,
    pub workflow_notes: String,
    pub config_notes: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AgentUiConfig {
    pub id: String,
    pub display_name: String,
    pub role: String,
    pub summary: String,
    pub avatar_label: String,
    pub enabled: bool,
    pub skills: Vec<String>,
    pub capabilities: Vec<String>,
    pub tools: Vec<String>,
    pub system_prompt: String,
    pub heartbeat_every: String,
    pub model: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StudioState {
    pub selected_project_id: String,
    pub projects: Vec<ProjectRecord>,
    pub agents: Vec<AgentUiConfig>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RenderedAgentConfig {
    pub agent_id: String,
    pub project_id: Option<String>,
    pub target: String,
    pub rendered_at: String,
    pub content: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ApplyAgentConfigResult {
    pub applied: bool,
    pub agent_id: String,
    pub project_id: Option<String>,
    pub target: String,
    pub rendered_at: String,
    pub message: String,
}

fn default_projects() -> Vec<ProjectRecord> {
    vec![
        ProjectRecord {
            id: "cto-core".to_string(),
            name: "Sigma 1".to_string(),
            summary: "Primary product workspace for the live Morgan experience.".to_string(),
            repository: Some("/Users/jonathon/5dlabs/cto".to_string()),
            prd_title: "Sigma 1".to_string(),
            prd_content: "Unify chat, voice, video, agents, and projects around one local Morgan experience.".to_string(),
            workflow_summary: "Morgan intake -> agent execution -> review -> apply.".to_string(),
            workflow_notes: "Track PRD decomposition, workflow steps, and local runtime checkpoints here.".to_string(),
            config_notes: "Kind-backed local OpenClaw stack with deterministic desktop-owned config.".to_string(),
        },
        ProjectRecord {
            id: "morgan-avatar".to_string(),
            name: "Morgan Avatar".to_string(),
            summary: "Voice and video interaction layer for Morgan over the shared ACP session.".to_string(),
            repository: Some("/Users/jonathon/5dlabs/cto/avatar".to_string()),
            prd_title: "Morgan Voice and Video".to_string(),
            prd_content: "Productionize the talking avatar with stable turn-taking, shared context, and clean call controls.".to_string(),
            workflow_summary: "Token server -> LiveKit room -> avatar worker -> Morgan ACP.".to_string(),
            workflow_notes: "Use this project when refining call UX, STT/TTS behavior, and avatar interactions.".to_string(),
            config_notes: "Project-scoped Morgan room/session shared across chat, voice, and video.".to_string(),
        },
    ]
}

fn normalize_state(mut state: StudioState) -> StudioState {
    for project in &mut state.projects {
        if project.id == "cto-core" && project.name == "CTO Core" {
            project.name = "Sigma 1".to_string();
            project.summary =
                "Primary product workspace for the live Morgan experience.".to_string();
            if project.prd_title == "CTO Desktop MVP" {
                project.prd_title = "Sigma 1".to_string();
            }
        }
    }

    state
}

fn default_agents() -> Vec<AgentUiConfig> {
    vec![
        AgentUiConfig {
            id: "morgan".to_string(),
            display_name: "Morgan".to_string(),
            role: "Intake & Orchestrator".to_string(),
            summary: "Primary conversational agent across chat, voice, and video.".to_string(),
            avatar_label: "MO".to_string(),
            enabled: true,
            skills: vec![
                "openclaw".to_string(),
                "acp".to_string(),
                "memory".to_string(),
                "workflow-routing".to_string(),
            ],
            capabilities: vec![
                "Conversation".to_string(),
                "Task decomposition".to_string(),
                "Delegation".to_string(),
                "Project context".to_string(),
            ],
            tools: vec![
                "codex".to_string(),
                "claude".to_string(),
                "github".to_string(),
                "linear".to_string(),
            ],
            system_prompt: "You are Morgan, the crisp and practical orchestrator for CTO. Keep responses concise, collaborative, and execution-oriented.".to_string(),
            heartbeat_every: "5m".to_string(),
            model: "anthropic/claude-sonnet-4-20250514".to_string(),
        },
        AgentUiConfig {
            id: "atlas".to_string(),
            display_name: "Atlas".to_string(),
            role: "Merge Gate".to_string(),
            summary: "Release and branch management agent.".to_string(),
            avatar_label: "AT".to_string(),
            enabled: true,
            skills: vec!["git".to_string(), "merge".to_string()],
            capabilities: vec!["Branch policy".to_string(), "Release hygiene".to_string()],
            tools: vec!["github".to_string()],
            system_prompt: "Atlas manages merge readiness and release gate policy.".to_string(),
            heartbeat_every: "15m".to_string(),
            model: "anthropic/claude-3-5-haiku-latest".to_string(),
        },
        AgentUiConfig {
            id: "stitch".to_string(),
            display_name: "Stitch".to_string(),
            role: "Code Review".to_string(),
            summary: "Automated PR review and quality signal.".to_string(),
            avatar_label: "ST".to_string(),
            enabled: true,
            skills: vec!["review".to_string(), "ci".to_string()],
            capabilities: vec!["PR review".to_string(), "Check synthesis".to_string()],
            tools: vec!["github".to_string()],
            system_prompt: "Stitch reviews code for bugs, regressions, and missing tests.".to_string(),
            heartbeat_every: "15m".to_string(),
            model: "anthropic/claude-3-5-haiku-latest".to_string(),
        },
        AgentUiConfig {
            id: "rex".to_string(),
            display_name: "Rex".to_string(),
            role: "Rust Implementation".to_string(),
            summary: "Backend implementation specialist.".to_string(),
            avatar_label: "RX".to_string(),
            enabled: true,
            skills: vec!["rust".to_string(), "cargo".to_string()],
            capabilities: vec!["Systems code".to_string(), "CLI work".to_string()],
            tools: vec!["cargo".to_string(), "github".to_string()],
            system_prompt: "Rex implements backend and Rust runtime changes.".to_string(),
            heartbeat_every: "15m".to_string(),
            model: "anthropic/claude-3-5-haiku-latest".to_string(),
        },
        AgentUiConfig {
            id: "blaze".to_string(),
            display_name: "Blaze".to_string(),
            role: "Frontend Implementation".to_string(),
            summary: "React and TypeScript implementation specialist.".to_string(),
            avatar_label: "BZ".to_string(),
            enabled: true,
            skills: vec!["react".to_string(), "typescript".to_string()],
            capabilities: vec!["UI implementation".to_string(), "State management".to_string()],
            tools: vec!["npm".to_string(), "github".to_string()],
            system_prompt: "Blaze builds frontend interfaces and interaction flows.".to_string(),
            heartbeat_every: "15m".to_string(),
            model: "anthropic/claude-3-5-haiku-latest".to_string(),
        },
        AgentUiConfig {
            id: "angie".to_string(),
            display_name: "Angie".to_string(),
            role: "Agent Architect".to_string(),
            summary: "OpenClaw-first specialist for agent systems and orchestration.".to_string(),
            avatar_label: "AN".to_string(),
            enabled: true,
            skills: vec![
                "openclaw-cli".to_string(),
                "openclaw-mcp-debugger".to_string(),
                "agent-team-orchestration".to_string(),
                "livekit-voice".to_string(),
                "elevenlabs-agents".to_string(),
                "elevenlabs-api".to_string(),
            ],
            capabilities: vec![
                "Agent platform design".to_string(),
                "Multi-agent routing".to_string(),
                "Tool/runtime integration".to_string(),
            ],
            tools: vec![
                "codex".to_string(),
                "claude".to_string(),
                "github".to_string(),
                "linear".to_string(),
            ],
            system_prompt: "Angie designs and evolves OpenClaw-first agent systems, orchestration patterns, and MCP-connected workflows.".to_string(),
            heartbeat_every: "15m".to_string(),
            model: "anthropic/claude-opus-4-6-20260205".to_string(),
        },
    ]
}

fn default_state() -> StudioState {
    let projects = default_projects();
    StudioState {
        selected_project_id: projects
            .first()
            .map(|project| project.id.clone())
            .unwrap_or_else(|| "cto-core".to_string()),
        projects,
        agents: default_agents(),
    }
}

fn load_state(db: &Database) -> Result<StudioState, AppError> {
    if let Some(raw) = db.get_config(STUDIO_STATE_KEY)? {
        let state = normalize_state(serde_json::from_str(&raw)?);
        if !state.projects.is_empty()
            && state
                .projects
                .iter()
                .any(|project| project.id == state.selected_project_id)
        {
            return Ok(state);
        }
    }

    Ok(normalize_state(default_state()))
}

fn save_state(db: &Database, state: &StudioState) -> Result<(), AppError> {
    let state = normalize_state(state.clone());
    if state.projects.is_empty() {
        return Err(AppError::ConfigError(
            "Studio state must contain at least one project".to_string(),
        ));
    }

    if !state
        .projects
        .iter()
        .any(|project| project.id == state.selected_project_id)
    {
        return Err(AppError::ConfigError(
            "Selected project must reference an existing project".to_string(),
        ));
    }

    let raw = serde_json::to_string(&state)?;
    db.set_config(STUDIO_STATE_KEY, &raw)?;
    Ok(())
}

fn render_agent_config_internal(
    state: &StudioState,
    agent_id: &str,
    project_id: Option<&str>,
) -> Result<RenderedAgentConfig, AppError> {
    let agent = state
        .agents
        .iter()
        .find(|candidate| candidate.id == agent_id)
        .ok_or_else(|| AppError::ConfigError(format!("Unknown agent id: {agent_id}")))?;
    let project =
        project_id.and_then(|id| state.projects.iter().find(|candidate| candidate.id == id));
    let rendered_at = Utc::now().to_rfc3339();
    let target = format!("local-runtime://openclaw/agents/{agent_id}");
    let payload = serde_json::json!({
        "agent": {
            "id": agent.id,
            "displayName": agent.display_name,
            "role": agent.role,
            "enabled": agent.enabled,
            "heartbeat": {
                "every": agent.heartbeat_every,
            },
            "model": {
                "primary": agent.model,
            },
            "skills": agent.skills,
            "capabilities": agent.capabilities,
            "tools": agent.tools,
            "prompt": agent.system_prompt,
        },
        "projectBinding": project.map(|value| {
            serde_json::json!({
                "id": value.id,
                "name": value.name,
                "summary": value.summary,
                "repository": value.repository,
                "workflowSummary": value.workflow_summary,
                "prdTitle": value.prd_title,
            })
        }),
        "generatedBy": "cto-desktop",
        "renderedAt": rendered_at,
    });

    Ok(RenderedAgentConfig {
        agent_id: agent_id.to_string(),
        project_id: project_id.map(ToOwned::to_owned),
        target,
        rendered_at,
        content: serde_json::to_string_pretty(&payload)?,
    })
}

#[tauri::command]
pub async fn studio_get_state(db: State<'_, Database>) -> Result<StudioState, AppError> {
    let state = load_state(db.inner())?;
    if db.get_config(STUDIO_STATE_KEY)?.is_none() {
        save_state(db.inner(), &state)?;
    }
    Ok(state)
}

#[tauri::command]
pub async fn studio_save_state(
    db: State<'_, Database>,
    state: StudioState,
) -> Result<StudioState, AppError> {
    save_state(db.inner(), &state)?;
    Ok(state)
}

#[tauri::command]
pub async fn studio_render_agent_config(
    db: State<'_, Database>,
    agent_id: String,
    project_id: Option<String>,
) -> Result<RenderedAgentConfig, AppError> {
    let state = load_state(db.inner())?;
    render_agent_config_internal(&state, &agent_id, project_id.as_deref())
}

#[tauri::command]
pub async fn studio_export_agent_config(
    db: State<'_, Database>,
    agent_id: String,
    project_id: Option<String>,
) -> Result<RenderedAgentConfig, AppError> {
    let state = load_state(db.inner())?;
    render_agent_config_internal(&state, &agent_id, project_id.as_deref())
}

#[tauri::command]
pub async fn studio_apply_agent_config(
    app: AppHandle,
    db: State<'_, Database>,
    agent_id: String,
    project_id: Option<String>,
) -> Result<ApplyAgentConfigResult, AppError> {
    let state = load_state(db.inner())?;
    let rendered = render_agent_config_internal(&state, &agent_id, project_id.as_deref())?;

    let app_data_dir = app.path().app_data_dir().map_err(|error| {
        AppError::CommandFailed(format!("Failed to resolve app data dir: {error}"))
    })?;
    let output_dir = app_data_dir.join("studio").join("generated");
    std::fs::create_dir_all(&output_dir)?;
    let file_path = output_dir.join(format!("{}-runtime.json", agent_id));
    std::fs::write(&file_path, &rendered.content)?;
    db.set_config(
        &format!("studio_applied_agent::{agent_id}"),
        &rendered.content,
    )?;

    Ok(ApplyAgentConfigResult {
        applied: true,
        agent_id,
        project_id,
        target: file_path.to_string_lossy().to_string(),
        rendered_at: rendered.rendered_at,
        message: "Generated runtime config saved for local application.".to_string(),
    })
}
