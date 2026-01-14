//! Play session management for Healer.
//!
//! Tracks active Play sessions with their full context:
//! - CTO config (expected tools per agent)
//! - Task list and dependencies
//! - Repository and service info
//! - Expected agents and their tool requirements

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, info, warn};

/// Expected tools for an agent from CTO config.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct AgentTools {
    /// Remote tools from tools-server
    #[serde(default)]
    pub remote: Vec<String>,
    /// Local MCP servers (filesystem, git, etc.)
    #[serde(default, rename = "localServers")]
    pub local_servers: HashMap<String, LocalServerConfig>,
}

/// Local MCP server configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LocalServerConfig {
    /// Whether this server is enabled
    #[serde(default)]
    pub enabled: bool,
    /// Tools provided by this server
    #[serde(default)]
    pub tools: Vec<String>,
}

/// Task definition from intake.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskInfo {
    /// Task ID
    pub id: String,
    /// Task title
    pub title: String,
    /// Agent hint (rex, blaze, bolt, etc.)
    #[serde(default)]
    pub agent_hint: Option<String>,
    /// Dependencies (task IDs)
    #[serde(default)]
    pub dependencies: Vec<String>,
    /// Priority
    #[serde(default)]
    pub priority: i32,
}

/// CTO config subset relevant to Healer.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct CtoConfig {
    /// Agent configurations with expected tools
    #[serde(default)]
    pub agents: HashMap<String, AgentConfig>,
}

/// Agent configuration from CTO config.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct AgentConfig {
    /// GitHub App for this agent
    #[serde(default, rename = "githubApp")]
    pub github_app: Option<String>,
    /// CLI type (claude, opencode, etc.)
    #[serde(default)]
    pub cli: Option<String>,
    /// Model to use
    #[serde(default)]
    pub model: Option<String>,
    /// Expected tools
    #[serde(default)]
    pub tools: AgentTools,
}

/// Request to start a new Play session.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StartSessionRequest {
    /// Play ID (usually task-id based)
    pub play_id: String,
    /// Repository URL
    pub repository: String,
    /// Service identifier
    #[serde(default)]
    pub service: Option<String>,
    /// CTO config for this play
    #[serde(default)]
    pub cto_config: CtoConfig,
    /// Tasks to be executed
    #[serde(default)]
    pub tasks: Vec<TaskInfo>,
    /// Namespace for `CodeRuns`
    #[serde(default = "default_namespace")]
    pub namespace: String,
}

fn default_namespace() -> String {
    "cto".to_string()
}

/// Response when starting a session.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StartSessionResponse {
    /// Status
    pub status: &'static str,
    /// Session ID (same as `play_id`)
    pub session_id: String,
    /// Message
    pub message: String,
}

/// An active Play session being monitored.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlaySession {
    /// Play ID
    pub play_id: String,
    /// Repository
    pub repository: String,
    /// Service identifier
    pub service: Option<String>,
    /// CTO config for this play
    pub cto_config: CtoConfig,
    /// Tasks in this play
    pub tasks: Vec<TaskInfo>,
    /// Namespace
    pub namespace: String,
    /// When the session started
    pub started_at: DateTime<Utc>,
    /// When the session was last updated
    pub last_updated: DateTime<Utc>,
    /// Detected issues in this session
    #[serde(default)]
    pub issues: Vec<SessionIssue>,
    /// Current status
    pub status: SessionStatus,
}

/// Status of a Play session.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Default)]
#[serde(rename_all = "snake_case")]
pub enum SessionStatus {
    /// Session is active and being monitored
    #[default]
    Active,
    /// Session completed successfully
    Completed,
    /// Session failed
    Failed,
    /// Session was cancelled
    Cancelled,
}

/// An issue detected during a Play session.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionIssue {
    /// When the issue was detected
    pub detected_at: DateTime<Utc>,
    /// Issue type
    pub issue_type: IssueType,
    /// Severity
    pub severity: IssueSeverity,
    /// Description
    pub description: String,
    /// Agent involved (if any)
    pub agent: Option<String>,
    /// Task involved (if any)
    pub task_id: Option<String>,
    /// Whether a remediation was spawned
    pub remediation_spawned: bool,
    /// GitHub issue URL (if created)
    pub github_issue: Option<String>,
}

/// Type of issue detected.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum IssueType {
    /// Pre-flight check failed (prompts or tools)
    PreFlightFailure,
    /// Tool inventory mismatch (A10)
    ToolMismatch,
    /// CTO config missing or invalid (A11)
    ConfigError,
    /// MCP server initialization failed (A12)
    McpInitFailed,
    /// General agent error from logs
    AgentError,
    /// Agent stuck (no progress)
    Stuck,
    /// Agent failure
    AgentFailure,
    /// Build failure
    BuildFailure,
    /// Test failure
    TestFailure,
    /// Language mismatch (wrong tools for language)
    LanguageMismatch,
    /// Other anomaly
    Other,
}

/// Severity of an issue.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum IssueSeverity {
    /// Critical - blocks everything
    Critical,
    /// High - needs immediate attention
    High,
    /// Medium - should be addressed
    Medium,
    /// Low - informational
    Low,
}

/// Session store for active Play sessions.
#[derive(Debug, Default)]
pub struct SessionStore {
    /// Active sessions by `play_id`
    sessions: RwLock<HashMap<String, PlaySession>>,
}

impl SessionStore {
    /// Create a new session store.
    #[must_use]
    pub fn new() -> Self {
        Self {
            sessions: RwLock::new(HashMap::new()),
        }
    }

    /// Start a new Play session.
    pub async fn start_session(&self, request: StartSessionRequest) -> PlaySession {
        let now = Utc::now();
        let session = PlaySession {
            play_id: request.play_id.clone(),
            repository: request.repository,
            service: request.service,
            cto_config: request.cto_config,
            tasks: request.tasks,
            namespace: request.namespace,
            started_at: now,
            last_updated: now,
            issues: Vec::new(),
            status: SessionStatus::Active,
        };

        info!(
            play_id = %session.play_id,
            repository = %session.repository,
            tasks = %session.tasks.len(),
            "Started new Play session"
        );

        let mut sessions = self.sessions.write().await;
        sessions.insert(session.play_id.clone(), session.clone());

        session
    }

    /// Get a session by `play_id`.
    pub async fn get_session(&self, play_id: &str) -> Option<PlaySession> {
        let sessions = self.sessions.read().await;
        sessions.get(play_id).cloned()
    }

    /// Get all active sessions.
    pub async fn get_active_sessions(&self) -> Vec<PlaySession> {
        let sessions = self.sessions.read().await;
        sessions
            .values()
            .filter(|s| s.status == SessionStatus::Active)
            .cloned()
            .collect()
    }

    /// Get all sessions (active and inactive).
    pub async fn get_all_sessions(&self) -> Vec<PlaySession> {
        let sessions = self.sessions.read().await;
        sessions.values().cloned().collect()
    }

    /// Update a session.
    pub async fn update_session(&self, session: PlaySession) {
        let mut sessions = self.sessions.write().await;
        sessions.insert(session.play_id.clone(), session);
    }

    /// Add an issue to a session.
    ///
    /// # Errors
    ///
    /// Returns an error if the session is not found.
    pub async fn add_issue(&self, play_id: &str, issue: SessionIssue) -> Result<(), String> {
        let mut sessions = self.sessions.write().await;
        if let Some(session) = sessions.get_mut(play_id) {
            info!(
                play_id = %play_id,
                issue_type = ?issue.issue_type,
                severity = ?issue.severity,
                "Added issue to session"
            );
            session.issues.push(issue);
            session.last_updated = Utc::now();
            Ok(())
        } else {
            warn!(play_id = %play_id, "Session not found, cannot add issue");
            Err(format!("Session {play_id} not found"))
        }
    }

    /// Mark a session as completed.
    pub async fn complete_session(&self, play_id: &str, success: bool) {
        let mut sessions = self.sessions.write().await;
        if let Some(session) = sessions.get_mut(play_id) {
            session.status = if success {
                SessionStatus::Completed
            } else {
                SessionStatus::Failed
            };
            session.last_updated = Utc::now();
            info!(
                play_id = %play_id,
                success = %success,
                issues = %session.issues.len(),
                "Completed Play session"
            );
        }
    }

    /// Remove old completed sessions (cleanup).
    pub async fn cleanup_old_sessions(&self, max_age_hours: i64) {
        let cutoff = Utc::now() - chrono::Duration::hours(max_age_hours);
        let mut sessions = self.sessions.write().await;

        let old_count = sessions.len();
        sessions.retain(|_, session| {
            session.status == SessionStatus::Active || session.last_updated > cutoff
        });

        let removed = old_count - sessions.len();
        if removed > 0 {
            debug!(removed = %removed, "Cleaned up old sessions");
        }
    }

    /// Get expected tools for an agent from the session's CTO config.
    pub async fn get_expected_tools(&self, play_id: &str, agent: &str) -> Option<AgentTools> {
        let sessions = self.sessions.read().await;
        sessions
            .get(play_id)
            .and_then(|s| s.cto_config.agents.get(agent))
            .map(|a| a.tools.clone())
    }
}

/// Thread-safe session store handle.
pub type SessionStoreHandle = Arc<SessionStore>;

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_session_lifecycle() {
        let store = SessionStore::new();

        // Start a session
        let request = StartSessionRequest {
            play_id: "test-play-1".to_string(),
            repository: "5dlabs/test".to_string(),
            service: Some("test-service".to_string()),
            cto_config: CtoConfig::default(),
            tasks: vec![TaskInfo {
                id: "1".to_string(),
                title: "Test task".to_string(),
                agent_hint: Some("rex".to_string()),
                dependencies: vec![],
                priority: 0,
            }],
            namespace: "cto".to_string(),
        };

        let session = store.start_session(request).await;
        assert_eq!(session.status, SessionStatus::Active);

        // Get session
        let retrieved = store.get_session("test-play-1").await;
        assert!(retrieved.is_some());

        // Add issue
        let result = store
            .add_issue(
                "test-play-1",
                SessionIssue {
                    detected_at: Utc::now(),
                    issue_type: IssueType::ToolMismatch,
                    severity: IssueSeverity::Critical,
                    description: "Tool brave_search missing".to_string(),
                    agent: Some("rex".to_string()),
                    task_id: Some("1".to_string()),
                    remediation_spawned: false,
                    github_issue: None,
                },
            )
            .await;
        assert!(result.is_ok());

        // Complete session
        store.complete_session("test-play-1", false).await;

        let completed = store.get_session("test-play-1").await.unwrap();
        assert_eq!(completed.status, SessionStatus::Failed);
        assert_eq!(completed.issues.len(), 1);
    }
}
