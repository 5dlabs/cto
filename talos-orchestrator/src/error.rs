use thiserror::Error;
use std::fmt;

use crate::state::InstallationState;

#[derive(Debug, Error)]
pub enum Error {
    #[error("Scaleway API error: {0}")]
    ScalewayApi(#[from] reqwest::Error),

    #[error("SSH error: {0}")]
    Ssh(String),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Configuration error: {0}")]
    Config(String),

    #[error("Configuration file error: {0}")]
    ConfigFile(#[from] config::ConfigError),

    #[error("Server not found: {0}")]
    ServerNotFound(String),

    #[error("Rescue mode timeout after {0:?}")]
    RescueModeTimeout(std::time::Duration),

    #[error("Talos boot timeout after {0:?}")]
    TalosBootTimeout(std::time::Duration),

    #[error("Installation failed at state {state}: {message}")]
    InstallationFailed {
        state: InstallationState,
        message: String,
    },

    #[error("SSH key error: {0}")]
    SshKey(String),
}

impl From<ssh2::Error> for Error {
    fn from(e: ssh2::Error) -> Self {
        Error::Ssh(e.to_string())
    }
}

impl From<tokio::task::JoinError> for Error {
    fn from(e: tokio::task::JoinError) -> Self {
        Error::Ssh(e.to_string())
    }
}

pub type Result<T> = std::result::Result<T, Error>;
