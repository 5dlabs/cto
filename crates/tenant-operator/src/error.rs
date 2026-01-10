//! Error types for the tenant operator

use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("Kubernetes API error: {0}")]
    Kube(#[from] kube::Error),

    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    #[error("YAML error: {0}")]
    Yaml(#[from] serde_yaml::Error),

    #[error("Namespace creation failed: {0}")]
    NamespaceCreation(String),

    #[error("RBAC setup failed: {0}")]
    RbacSetup(String),

    #[error("ExternalSecret creation failed: {0}")]
    ExternalSecret(String),

    #[error("ArgoCD Application creation failed: {0}")]
    ArgoApp(String),

    #[error("OpenBao error: {0}")]
    OpenBao(String),

    #[error("Invalid tenant configuration: {0}")]
    InvalidConfig(String),

    #[error("Finalizer error: {0}")]
    Finalizer(#[source] Box<kube::runtime::finalizer::Error<Error>>),
}

pub type Result<T> = std::result::Result<T, Error>;
