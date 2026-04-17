//! `ManagedRepo` Custom Resource Definition for intake pipeline
//!
//! Registry entry describing a repository under CTO intake management.
//! The self-hosted GitHub Actions intake runner watches these resources
//! to decide which repositories to scan for PRD changes.
//!
//! Scope: cluster-scoped — a repository is a global fact (it has one
//! canonical `owner/name`), and scoping it to a namespace would force
//! duplicate entries for every consumer namespace.

use kube::CustomResource;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

fn default_prd_path() -> String {
    "docs/prd.md".to_string()
}

fn default_true() -> bool {
    true
}

fn default_voice() -> String {
    "alloy".to_string()
}

/// Briefing generation configuration for a `ManagedRepo`.
#[derive(Clone, Debug, Deserialize, Serialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct BriefingConfig {
    /// Whether briefing generation is enabled for this repo.
    #[serde(default = "default_true")]
    pub enabled: bool,

    /// TTS voice to use for the audio briefing. Defaults to `alloy`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub voice: Option<String>,
}

impl Default for BriefingConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            voice: Some(default_voice()),
        }
    }
}

/// `ManagedRepo` CRD — registry of repositories under intake management.
///
/// Group: `agents.platform`, Version: `v1alpha1`, Kind: `ManagedRepo`.
/// Cluster-scoped.
#[derive(CustomResource, Clone, Debug, Deserialize, Serialize, JsonSchema)]
#[kube(group = "agents.platform", version = "v1alpha1", kind = "ManagedRepo")]
#[kube(status = "ManagedRepoStatus")]
#[kube(shortname = "mrepo", shortname = "mrepos", shortname = "managedrepos")]
#[kube(printcolumn = r#"{"name":"Owner","type":"string","jsonPath":".spec.owner"}"#)]
#[kube(printcolumn = r#"{"name":"Name","type":"string","jsonPath":".spec.name"}"#)]
#[kube(
    printcolumn = r#"{"name":"LastIntakeTime","type":"date","jsonPath":".status.lastIntakeTime"}"#
)]
#[kube(printcolumn = r#"{"name":"Enabled","type":"boolean","jsonPath":".spec.enabled"}"#)]
#[serde(rename_all = "camelCase")]
pub struct ManagedRepoSpec {
    /// GitHub owner (user or org), e.g. `5dlabs`.
    pub owner: String,

    /// GitHub repository name, e.g. `sigma-1`.
    pub name: String,

    /// Path to the PRD markdown inside the repo. Defaults to `docs/prd.md`.
    #[serde(default = "default_prd_path")]
    pub prd_path: String,

    /// Briefing generation configuration.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub briefing: Option<BriefingConfig>,

    /// Master switch — disable to pause intake without deleting the resource.
    #[serde(default = "default_true")]
    pub enabled: bool,
}

/// Status for a `ManagedRepo` resource.
#[derive(Clone, Debug, Default, Deserialize, Serialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct ManagedRepoStatus {
    /// Git ref/sha of the most recent successful intake.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub last_intake_ref: Option<String>,

    /// RFC3339 timestamp of the most recent successful intake.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub last_intake_time: Option<String>,

    /// Total number of successful intakes recorded for this repo.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub total_intakes: Option<u32>,

    /// Conditions for detailed status tracking.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub conditions: Option<Vec<ManagedRepoCondition>>,
}

/// Condition for detailed `ManagedRepo` status (mirrors `BoltRunCondition`).
#[derive(Clone, Debug, Deserialize, Serialize, JsonSchema, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct ManagedRepoCondition {
    /// Type of condition (e.g. `Ready`, `IntakeSucceeded`).
    #[serde(rename = "type")]
    pub condition_type: String,

    /// Status of the condition: `True`, `False`, or `Unknown`.
    pub status: String,

    /// Last time the condition transitioned (RFC3339).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub last_transition_time: Option<String>,

    /// Reason for the condition's last transition.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub reason: Option<String>,

    /// Human-readable message about the condition.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub message: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn managed_repo_spec_round_trips() {
        let spec = ManagedRepoSpec {
            owner: "5dlabs".to_string(),
            name: "sigma-1".to_string(),
            prd_path: default_prd_path(),
            briefing: Some(BriefingConfig::default()),
            enabled: true,
        };
        let json = serde_json::to_string(&spec).unwrap();
        assert!(json.contains("\"owner\":\"5dlabs\""));
        assert!(json.contains("\"name\":\"sigma-1\""));
        assert!(json.contains("\"prdPath\":\"docs/prd.md\""));
        assert!(json.contains("\"enabled\":true"));
    }

    #[test]
    fn briefing_config_defaults_alloy() {
        let b = BriefingConfig::default();
        assert!(b.enabled);
        assert_eq!(b.voice.as_deref(), Some("alloy"));
    }
}
