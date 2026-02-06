//! Application state management

use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::process::Child;
use tokio::sync::RwLock;
use tokio::sync::Mutex;
use std::collections::HashMap;

/// Global application state
#[derive(Debug, Default)]
pub struct AppState {
    pub setup: Arc<RwLock<SetupState>>,
    pub cluster: Arc<RwLock<ClusterState>>,
    pub mcp_sessions: Arc<Mutex<HashMap<String, Child>>>,
}

impl AppState {
    pub fn new() -> Self {
        Self::default()
    }
}

/// Setup wizard state
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SetupState {
    pub current_step: usize,
    pub completed: bool,
    pub stack_selection: Option<StackSelection>,
    pub api_keys_configured: ApiKeysConfigured,
    pub docker_verified: bool,
    pub cluster_created: bool,
}

/// Stack selection (Nova or Grizz)
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum StackSelection {
    Nova,  // TypeScript/Effect
    Grizz, // Rust/Axum
}

/// API keys configuration status
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ApiKeysConfigured {
    pub anthropic: bool,
    pub openai: bool,
    pub github: bool,
}

/// Cluster state
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ClusterState {
    pub name: Option<String>,
    pub status: ClusterStatus,
    pub kubeconfig_path: Option<String>,
}

/// Cluster status
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum ClusterStatus {
    #[default]
    NotCreated,
    Creating,
    Running,
    Stopping,
    Stopped,
    Error,
}
