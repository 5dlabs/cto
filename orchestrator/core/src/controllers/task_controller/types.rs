use super::config::ControllerConfig;
use kube::Client;
use std::sync::Arc;

// Error type for the controller
#[derive(Debug, thiserror::Error)]
#[allow(clippy::enum_variant_names)]
pub enum Error {
    #[error("Kubernetes API error: {0}")]
    KubeError(#[from] kube::Error),

    #[error("Missing object key")]
    MissingObjectKey,

    #[error("Serialization error: {0}")]
    SerializationError(#[from] serde_json::Error),

    #[error("Task configuration error: {0}")]
    ConfigError(String),
}

pub type Result<T, E = Error> = std::result::Result<T, E>;

// Context shared across controller operations
#[derive(Clone)]
pub(crate) struct Context {
    pub client: Client,
    pub namespace: String,
    pub config: Arc<ControllerConfig>,
}

// Finalizer names for cleanup
pub(crate) const DOCS_FINALIZER_NAME: &str = "docsruns.orchestrator.io/finalizer";
pub(crate) const CODE_FINALIZER_NAME: &str = "coderuns.orchestrator.io/finalizer";

// Helper functions for SSH and GitHub token secret names
pub fn ssh_secret_name(github_user: &str) -> String {
    format!("github-ssh-{github_user}")
}

pub fn github_token_secret_name(github_user: &str) -> String {
    format!("github-token-{github_user}")
}
