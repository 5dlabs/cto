//! Play workflow stages and transitions.

use serde::{Deserialize, Serialize};
use std::time::Duration;

/// Stage timeout - 30 minutes is the target, anything longer is suspicious.
pub const STAGE_TIMEOUT: Duration = Duration::from_secs(30 * 60);

/// Canonical play workflow stages.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Stage {
    /// Initial state - workflow started
    Pending,
    /// Rex writing code
    ImplementationInProgress,
    /// Cleo reviewing code
    QualityInProgress,
    /// Cipher security scan
    SecurityInProgress,
    /// Tess running tests
    TestingInProgress,
    /// Atlas merging PR
    WaitingAtlasIntegration,
    /// PR merged to main
    WaitingPrMerged,
    /// Workflow finished successfully
    Completed,
    /// Workflow failed
    Failed,
}

impl Stage {
    /// Get the agent responsible for this stage.
    #[must_use]
    pub fn agent(&self) -> Option<&'static str> {
        match self {
            Self::ImplementationInProgress => Some("Rex"),
            Self::QualityInProgress => Some("Cleo"),
            Self::SecurityInProgress => Some("Cipher"),
            Self::TestingInProgress => Some("Tess"),
            Self::WaitingAtlasIntegration => Some("Atlas"),
            _ => None,
        }
    }

    /// Get the timeout for this stage.
    #[must_use]
    pub fn timeout(&self) -> Duration {
        STAGE_TIMEOUT
    }

    /// Check if this is a terminal stage.
    #[must_use]
    pub fn is_terminal(&self) -> bool {
        matches!(self, Self::Completed | Self::Failed)
    }

    /// Check if this stage involves an agent.
    #[must_use]
    pub fn has_agent(&self) -> bool {
        self.agent().is_some()
    }

    /// Get the next expected stage in the workflow.
    #[must_use]
    pub fn next(&self) -> Option<Self> {
        match self {
            Self::Pending => Some(Self::ImplementationInProgress),
            Self::ImplementationInProgress => Some(Self::QualityInProgress),
            Self::QualityInProgress => Some(Self::SecurityInProgress),
            Self::SecurityInProgress => Some(Self::TestingInProgress),
            Self::TestingInProgress => Some(Self::WaitingAtlasIntegration),
            Self::WaitingAtlasIntegration => Some(Self::WaitingPrMerged),
            Self::WaitingPrMerged => Some(Self::Completed),
            Self::Completed | Self::Failed => None,
        }
    }

    /// Get the display name for this stage.
    #[must_use]
    pub fn display_name(&self) -> &'static str {
        match self {
            Self::Pending => "Pending",
            Self::ImplementationInProgress => "Implementation",
            Self::QualityInProgress => "Quality",
            Self::SecurityInProgress => "Security",
            Self::TestingInProgress => "Testing",
            Self::WaitingAtlasIntegration => "Atlas Integration",
            Self::WaitingPrMerged => "Waiting Merge",
            Self::Completed => "Completed",
            Self::Failed => "Failed",
        }
    }

    /// Parse a stage from a `ConfigMap` stage string.
    #[must_use]
    pub fn from_configmap_value(value: &str) -> Option<Self> {
        match value.to_lowercase().as_str() {
            "pending" => Some(Self::Pending),
            "implementation-in-progress" | "implementation" => {
                Some(Self::ImplementationInProgress)
            }
            "quality-in-progress" | "quality" => Some(Self::QualityInProgress),
            "security-in-progress" | "security" => Some(Self::SecurityInProgress),
            "testing-in-progress" | "testing" => Some(Self::TestingInProgress),
            "waiting-atlas-integration" | "atlas" => Some(Self::WaitingAtlasIntegration),
            "waiting-pr-merged" | "merge" => Some(Self::WaitingPrMerged),
            "completed" | "complete" | "done" => Some(Self::Completed),
            "failed" | "error" => Some(Self::Failed),
            _ => None,
        }
    }
}

impl std::fmt::Display for Stage {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.display_name())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_stage_agent() {
        assert_eq!(Stage::ImplementationInProgress.agent(), Some("Rex"));
        assert_eq!(Stage::QualityInProgress.agent(), Some("Cleo"));
        assert_eq!(Stage::SecurityInProgress.agent(), Some("Cipher"));
        assert_eq!(Stage::TestingInProgress.agent(), Some("Tess"));
        assert_eq!(Stage::WaitingAtlasIntegration.agent(), Some("Atlas"));
        assert_eq!(Stage::Pending.agent(), None);
        assert_eq!(Stage::Completed.agent(), None);
    }

    #[test]
    fn test_stage_timeout() {
        assert_eq!(Stage::ImplementationInProgress.timeout(), STAGE_TIMEOUT);
        assert_eq!(STAGE_TIMEOUT.as_secs(), 30 * 60);
    }

    #[test]
    fn test_stage_next() {
        assert_eq!(Stage::Pending.next(), Some(Stage::ImplementationInProgress));
        assert_eq!(
            Stage::ImplementationInProgress.next(),
            Some(Stage::QualityInProgress)
        );
        assert_eq!(Stage::Completed.next(), None);
        assert_eq!(Stage::Failed.next(), None);
    }

    #[test]
    fn test_from_configmap_value() {
        assert_eq!(
            Stage::from_configmap_value("implementation-in-progress"),
            Some(Stage::ImplementationInProgress)
        );
        assert_eq!(
            Stage::from_configmap_value("quality-in-progress"),
            Some(Stage::QualityInProgress)
        );
        assert_eq!(
            Stage::from_configmap_value("completed"),
            Some(Stage::Completed)
        );
        assert_eq!(Stage::from_configmap_value("unknown"), None);
    }
}

