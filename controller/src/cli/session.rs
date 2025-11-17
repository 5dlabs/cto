//! Session Manager
//!
//! Manages session state and persistence across different CLI types.
//! Handles state transitions and maintains context between CLI executions.

use crate::cli::types::{CLIType, UniversalConfig};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

/// Session state for a CLI execution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionState {
    /// Unique session ID
    pub id: String,
    /// CLI type used for this session
    pub cli_type: CLIType,
    /// Universal configuration
    pub universal_config: UniversalConfig,
    /// Session start time
    pub created_at: chrono::DateTime<chrono::Utc>,
    /// Last activity time
    pub last_active: chrono::DateTime<chrono::Utc>,
    /// CLI-specific state data
    pub cli_specific_state: serde_json::Value,
    /// Execution history
    pub execution_history: Vec<ExecutionRecord>,
    /// Session status
    pub status: SessionStatus,
}

/// Execution record for tracking session activity
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionRecord {
    /// Execution timestamp
    pub timestamp: chrono::DateTime<chrono::Utc>,
    /// Task that was executed
    pub task: String,
    /// Execution result
    pub result: ExecutionResult,
    /// Duration in milliseconds
    pub duration_ms: u64,
}

/// Execution result summary
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionResult {
    /// Success status
    pub success: bool,
    /// Exit code if available
    pub exit_code: Option<i32>,
    /// Key output/error messages
    pub key_messages: Vec<String>,
}

/// Session status
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum SessionStatus {
    /// Session is active and ready
    Active,
    /// Session is executing a task
    Executing,
    /// Session completed successfully
    Completed,
    /// Session failed
    Failed,
    /// Session was terminated
    Terminated,
}

/// Session persistence interface
#[async_trait]
pub trait SessionPersistence: Send + Sync {
    /// Save session state
    async fn save_session(&self, session: &SessionState) -> Result<()>;

    /// Load session state
    async fn load_session(&self, session_id: &str) -> Result<Option<SessionState>>;

    /// List all active sessions
    async fn list_sessions(&self) -> Result<Vec<SessionState>>;

    /// Delete session
    async fn delete_session(&self, session_id: &str) -> Result<()>;

    /// Clean up old sessions
    async fn cleanup_sessions(&self, max_age_hours: u64) -> Result<usize>;
}

/// In-memory session persistence (for development/testing)
pub struct MemorySessionPersistence {
    sessions: Arc<RwLock<HashMap<String, SessionState>>>,
}

impl Default for MemorySessionPersistence {
    fn default() -> Self {
        Self::new()
    }
}

impl MemorySessionPersistence {
    #[must_use]
    pub fn new() -> Self {
        Self {
            sessions: Arc::new(RwLock::new(HashMap::new())),
        }
    }
}

#[async_trait]
impl SessionPersistence for MemorySessionPersistence {
    async fn save_session(&self, session: &SessionState) -> Result<()> {
        let mut sessions = self.sessions.write().await;
        sessions.insert(session.id.clone(), session.clone());
        Ok(())
    }

    async fn load_session(&self, session_id: &str) -> Result<Option<SessionState>> {
        let sessions = self.sessions.read().await;
        Ok(sessions.get(session_id).cloned())
    }

    async fn list_sessions(&self) -> Result<Vec<SessionState>> {
        let sessions = self.sessions.read().await;
        Ok(sessions.values().cloned().collect())
    }

    async fn delete_session(&self, session_id: &str) -> Result<()> {
        let mut sessions = self.sessions.write().await;
        sessions.remove(session_id);
        Ok(())
    }

    async fn cleanup_sessions(&self, max_age_hours: u64) -> Result<usize> {
        let mut sessions = self.sessions.write().await;
        let cutoff = chrono::Utc::now()
            - chrono::Duration::hours(i64::try_from(max_age_hours).unwrap_or(i64::MAX));
        let initial_count = sessions.len();

        sessions.retain(|_, session| session.last_active > cutoff);

        Ok(initial_count - sessions.len())
    }
}

/// Session manager for handling CLI sessions
pub struct SessionManager {
    /// Session persistence layer
    persistence: Box<dyn SessionPersistence>,
    /// Active sessions cache
    active_sessions: Arc<RwLock<HashMap<String, SessionState>>>,
}

impl Default for SessionManager {
    fn default() -> Self {
        Self::new()
    }
}

impl SessionManager {
    /// Create a new session manager with memory persistence
    #[must_use]
    pub fn new() -> Self {
        Self {
            persistence: Box::new(MemorySessionPersistence::new()),
            active_sessions: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Create a new session
    pub async fn create_session(
        &self,
        cli_type: CLIType,
        universal_config: UniversalConfig,
    ) -> Result<String> {
        let session_id = format!("session_{}", uuid::Uuid::new_v4().simple());
        let now = chrono::Utc::now();

        let session = SessionState {
            id: session_id.clone(),
            cli_type,
            universal_config,
            created_at: now,
            last_active: now,
            cli_specific_state: serde_json::json!({}),
            execution_history: vec![],
            status: SessionStatus::Active,
        };

        // Save to persistence
        self.persistence.save_session(&session).await?;

        // Cache in memory
        let mut active = self.active_sessions.write().await;
        active.insert(session_id.clone(), session);

        Ok(session_id)
    }

    /// Get session by ID
    pub async fn get_session(&self, session_id: &str) -> Result<Option<SessionState>> {
        // Check cache first
        {
            let active = self.active_sessions.read().await;
            if let Some(session) = active.get(session_id) {
                return Ok(Some(session.clone()));
            }
        }

        // Load from persistence
        if let Some(session) = self.persistence.load_session(session_id).await? {
            // Cache it
            let mut active = self.active_sessions.write().await;
            active.insert(session_id.to_string(), session.clone());
            Ok(Some(session))
        } else {
            Ok(None)
        }
    }

    /// Update session status
    pub async fn update_session_status(
        &self,
        session_id: &str,
        status: SessionStatus,
    ) -> Result<()> {
        let mut session = self
            .get_session(session_id)
            .await?
            .ok_or_else(|| SessionError::SessionNotFound(session_id.to_string()))?;

        session.status = status;
        session.last_active = chrono::Utc::now();

        self.persistence.save_session(&session).await?;
        self.update_cache(&session).await;

        Ok(())
    }

    /// Record execution in session history
    pub async fn record_execution(
        &self,
        session_id: &str,
        task: &str,
        result: ExecutionResult,
        duration_ms: u64,
    ) -> Result<()> {
        let mut session = self
            .get_session(session_id)
            .await?
            .ok_or_else(|| SessionError::SessionNotFound(session_id.to_string()))?;

        let record = ExecutionRecord {
            timestamp: chrono::Utc::now(),
            task: task.to_string(),
            result,
            duration_ms,
        };

        session.execution_history.push(record);
        session.last_active = chrono::Utc::now();

        self.persistence.save_session(&session).await?;
        self.update_cache(&session).await;

        Ok(())
    }

    /// Update CLI-specific state
    pub async fn update_cli_state(&self, session_id: &str, state: serde_json::Value) -> Result<()> {
        let mut session = self
            .get_session(session_id)
            .await?
            .ok_or_else(|| SessionError::SessionNotFound(session_id.to_string()))?;

        session.cli_specific_state = state;
        session.last_active = chrono::Utc::now();

        self.persistence.save_session(&session).await?;
        self.update_cache(&session).await;

        Ok(())
    }

    /// Transition session to different CLI type
    pub async fn transition_cli(&self, session_id: &str, new_cli_type: CLIType) -> Result<()> {
        let mut session = self
            .get_session(session_id)
            .await?
            .ok_or_else(|| SessionError::SessionNotFound(session_id.to_string()))?;

        // Record the transition in history
        let transition_record = ExecutionRecord {
            timestamp: chrono::Utc::now(),
            task: format!(
                "CLI transition: {:?} → {:?}",
                session.cli_type, new_cli_type
            ),
            result: ExecutionResult {
                success: true,
                exit_code: None,
                key_messages: vec![format!(
                    "Transitioned from {:?} to {:?}",
                    session.cli_type, new_cli_type
                )],
            },
            duration_ms: 0,
        };

        session.cli_type = new_cli_type;
        session.execution_history.push(transition_record);
        session.last_active = chrono::Utc::now();

        self.persistence.save_session(&session).await?;
        self.update_cache(&session).await;

        Ok(())
    }

    /// List all active sessions
    pub async fn list_active_sessions(&self) -> Result<Vec<SessionState>> {
        let active = self.active_sessions.read().await;
        Ok(active.values().cloned().collect())
    }

    /// Clean up old sessions
    pub async fn cleanup_old_sessions(&self, max_age_hours: u64) -> Result<usize> {
        // Clean up persistence
        let cleaned = self.persistence.cleanup_sessions(max_age_hours).await?;

        // Clean up cache
        let mut active = self.active_sessions.write().await;
        let cutoff = chrono::Utc::now()
            - chrono::Duration::hours(i64::try_from(max_age_hours).unwrap_or(i64::MAX));
        active.retain(|_, session| session.last_active > cutoff);

        Ok(cleaned)
    }

    /// Update cached session
    async fn update_cache(&self, session: &SessionState) {
        let mut active = self.active_sessions.write().await;
        active.insert(session.id.clone(), session.clone());
    }
}

/// Session-specific errors
#[derive(Debug, thiserror::Error)]
pub enum SessionError {
    #[error("Session not found: {0}")]
    SessionNotFound(String),

    #[error("Session persistence error: {0}")]
    PersistenceError(String),

    #[error("Session serialization error: {0}")]
    SerializationError(String),

    #[error("Invalid session state: {0}")]
    InvalidState(String),
}

pub type Result<T> = std::result::Result<T, SessionError>;

// Convert serde_json errors
impl From<serde_json::Error> for SessionError {
    fn from(err: serde_json::Error) -> Self {
        SessionError::SerializationError(err.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cli::types::{AgentConfig, ContextConfig, SettingsConfig};

    #[tokio::test]
    async fn test_session_creation() {
        let manager = SessionManager::new();

        let config = UniversalConfig {
            context: ContextConfig {
                project_name: "Test".to_string(),
                project_description: "Test project".to_string(),
                architecture_notes: String::new(),
                constraints: vec![],
            },
            tools: vec![],
            settings: SettingsConfig {
                model: "gpt-4".to_string(),
                temperature: 0.7,
                max_tokens: 1000,
                timeout: 60,
                sandbox_mode: "read-only".to_string(),
            },
            agent: AgentConfig {
                role: "developer".to_string(),
                capabilities: vec![],
                instructions: "Test instructions".to_string(),
            },
            mcp_config: None,
        };

        let session_id = manager
            .create_session(CLIType::Codex, config)
            .await
            .unwrap();

        let session = manager.get_session(&session_id).await.unwrap().unwrap();
        assert_eq!(session.cli_type, CLIType::Codex);
        assert_eq!(session.status, SessionStatus::Active);
    }

    #[tokio::test]
    async fn test_session_status_update() {
        let manager = SessionManager::new();

        let config = UniversalConfig {
            context: ContextConfig {
                project_name: "Test".to_string(),
                project_description: "Test project".to_string(),
                architecture_notes: String::new(),
                constraints: vec![],
            },
            tools: vec![],
            settings: SettingsConfig {
                model: "gpt-4".to_string(),
                temperature: 0.7,
                max_tokens: 1000,
                timeout: 60,
                sandbox_mode: "read-only".to_string(),
            },
            agent: AgentConfig {
                role: "developer".to_string(),
                capabilities: vec![],
                instructions: "Test instructions".to_string(),
            },
            mcp_config: None,
        };

        let session_id = manager
            .create_session(CLIType::Claude, config)
            .await
            .unwrap();

        manager
            .update_session_status(&session_id, SessionStatus::Executing)
            .await
            .unwrap();

        let session = manager.get_session(&session_id).await.unwrap().unwrap();
        assert_eq!(session.status, SessionStatus::Executing);
    }

    #[tokio::test]
    async fn test_cli_transition() {
        let manager = SessionManager::new();

        let config = UniversalConfig {
            context: ContextConfig {
                project_name: "Test".to_string(),
                project_description: "Test project".to_string(),
                architecture_notes: String::new(),
                constraints: vec![],
            },
            tools: vec![],
            settings: SettingsConfig {
                model: "gpt-4".to_string(),
                temperature: 0.7,
                max_tokens: 1000,
                timeout: 60,
                sandbox_mode: "read-only".to_string(),
            },
            agent: AgentConfig {
                role: "developer".to_string(),
                capabilities: vec![],
                instructions: "Test instructions".to_string(),
            },
            mcp_config: None,
        };

        let session_id = manager
            .create_session(CLIType::Claude, config)
            .await
            .unwrap();

        manager
            .transition_cli(&session_id, CLIType::Codex)
            .await
            .unwrap();

        let session = manager.get_session(&session_id).await.unwrap().unwrap();
        assert_eq!(session.cli_type, CLIType::Codex);
        assert_eq!(session.execution_history.len(), 1);
        assert!(session.execution_history[0].task.contains("CLI transition"));
    }
}
