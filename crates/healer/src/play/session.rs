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
    ///
    /// Note: This method always inserts, potentially overwriting existing sessions.
    /// For concurrent-safe insertion that rejects duplicates, use [`try_start_session`].
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

    /// Atomically try to start a new Play session, rejecting if one already exists.
    ///
    /// This method holds the write lock during both the existence check and insertion,
    /// preventing TOCTOU race conditions where concurrent requests could both pass
    /// the duplicate check.
    ///
    /// # Errors
    ///
    /// Returns `Err(existing_session)` if a session with the same `play_id` already
    /// exists and is active. The returned error contains the existing session data.
    pub async fn try_start_session(
        &self,
        request: StartSessionRequest,
    ) -> Result<PlaySession, PlaySession> {
        let mut sessions = self.sessions.write().await;

        // Check for existing active session while holding the write lock
        if let Some(existing) = sessions.get(&request.play_id) {
            if existing.status == SessionStatus::Active {
                warn!(
                    play_id = %request.play_id,
                    "Rejecting duplicate session start - active session already exists"
                );
                return Err(existing.clone());
            }
            // Session exists but is not active - allow overwrite
            debug!(
                play_id = %request.play_id,
                status = ?existing.status,
                "Existing session is not active, allowing new session"
            );
        }

        // Create and insert the new session
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
            "Started new Play session (atomic)"
        );

        sessions.insert(session.play_id.clone(), session.clone());
        Ok(session)
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

    #[tokio::test]
    async fn test_try_start_session_rejects_duplicate() {
        let store = SessionStore::new();

        let request = StartSessionRequest {
            play_id: "test-play-dup".to_string(),
            repository: "5dlabs/test".to_string(),
            service: None,
            cto_config: CtoConfig::default(),
            tasks: vec![],
            namespace: "cto".to_string(),
        };

        // First start should succeed
        let result = store.try_start_session(request.clone()).await;
        assert!(result.is_ok());

        // Second start with same play_id should fail
        let result2 = store.try_start_session(request).await;
        assert!(result2.is_err());
    }

    #[tokio::test]
    async fn test_try_start_session_allows_overwrite_of_completed() {
        let store = SessionStore::new();

        let request = StartSessionRequest {
            play_id: "test-play-completed".to_string(),
            repository: "5dlabs/test".to_string(),
            service: None,
            cto_config: CtoConfig::default(),
            tasks: vec![],
            namespace: "cto".to_string(),
        };

        // Start initial session
        let result = store.try_start_session(request.clone()).await;
        assert!(result.is_ok());

        // Complete the session
        store.complete_session("test-play-completed", true).await;

        // Starting new session with same ID should succeed (old one is completed)
        let result2 = store.try_start_session(request).await;
        assert!(result2.is_ok());
    }

    #[tokio::test]
    async fn test_try_start_session_concurrent_race_protection() {
        use std::sync::atomic::{AtomicUsize, Ordering};

        let store = Arc::new(SessionStore::new());
        let success_count = Arc::new(AtomicUsize::new(0));
        let failure_count = Arc::new(AtomicUsize::new(0));

        // Spawn 10 concurrent tasks all trying to start the same session
        let mut handles = vec![];
        for i in 0..10 {
            let store = Arc::clone(&store);
            let success = Arc::clone(&success_count);
            let failure = Arc::clone(&failure_count);

            handles.push(tokio::spawn(async move {
                let request = StartSessionRequest {
                    play_id: "race-test-session".to_string(),
                    repository: format!("5dlabs/test-{i}"),
                    service: None,
                    cto_config: CtoConfig::default(),
                    tasks: vec![],
                    namespace: "cto".to_string(),
                };

                match store.try_start_session(request).await {
                    Ok(_) => success.fetch_add(1, Ordering::SeqCst),
                    Err(_) => failure.fetch_add(1, Ordering::SeqCst),
                };
            }));
        }

        // Wait for all tasks to complete
        for handle in handles {
            handle.await.unwrap();
        }

        // Exactly one should succeed, 9 should fail
        assert_eq!(success_count.load(Ordering::SeqCst), 1);
        assert_eq!(failure_count.load(Ordering::SeqCst), 9);

        // Verify only one session exists
        let all_sessions = store.get_all_sessions().await;
        assert_eq!(all_sessions.len(), 1);
    }
}
