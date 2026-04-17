//! `PRD` Custom Resource Definition for intake pipeline
//!
//! Represents a Product Requirements Document ingested from a managed
//! repository. The self-hosted GitHub Actions intake runner creates
//! `PRD` resources when it detects new or changed PRD markdown in a
//! `ManagedRepo`; downstream controllers then drive the briefing /
//! documentation generation pipeline.

use kube::CustomResource;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

fn default_prd_path() -> String {
    "docs/prd.md".to_string()
}

/// `PRD` CRD — one instance per (repo, contentHash) ingested PRD.
///
/// Group: `agents.platform`, Version: `v1alpha1`, Kind: `PRD`.
#[derive(CustomResource, Clone, Debug, Deserialize, Serialize, JsonSchema)]
#[kube(group = "agents.platform", version = "v1alpha1", kind = "PRD")]
#[kube(namespaced)]
#[kube(status = "PrdStatus")]
#[kube(shortname = "prd", shortname = "prds")]
#[kube(printcolumn = r#"{"name":"Repo","type":"string","jsonPath":".spec.repo"}"#)]
#[kube(printcolumn = r#"{"name":"Phase","type":"string","jsonPath":".status.phase"}"#)]
#[kube(printcolumn = r#"{"name":"Ref","type":"string","jsonPath":".spec.ref"}"#)]
#[kube(printcolumn = r#"{"name":"Age","type":"date","jsonPath":".metadata.creationTimestamp"}"#)]
#[serde(rename_all = "camelCase")]
pub struct PrdSpec {
    /// Source repository in `owner/name` form, e.g. `5dlabs/sigma-1`.
    pub repo: String,

    /// Git ref or SHA the PRD was extracted from.
    #[serde(rename = "ref")]
    pub git_ref: String,

    /// Path to the PRD markdown inside the repo. Defaults to `docs/prd.md`.
    #[serde(default = "default_prd_path")]
    pub prd_path: String,

    /// SHA-256 of the PRD content — used by the intake runner for dedup.
    pub content_hash: String,

    /// Optional inline markdown. Callers should omit for large PRDs and
    /// rely on `repo` + `ref` + `prdPath` to fetch on demand.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub content: Option<String>,
}

/// Status for a `PRD` resource.
#[derive(Clone, Debug, Default, Deserialize, Serialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct PrdStatus {
    /// Current phase: `Pending`, `Analyzing`, `BriefingGenerated`,
    /// `DocsGenerated`, `Complete`, or `Failed`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub phase: Option<String>,

    /// Content hash the controller last reconciled against.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub observed_content_hash: Option<String>,

    /// URL of the generated briefing artifact (audio/markdown/etc.).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub briefing_artifact_url: Option<String>,

    /// RFC3339 timestamp of the last phase transition.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub last_transition_time: Option<String>,

    /// Human-readable status message.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub message: Option<String>,

    /// Conditions for detailed status tracking.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub conditions: Option<Vec<PrdCondition>>,
}

/// Condition for detailed `PRD` status (mirrors `BoltRunCondition`).
#[derive(Clone, Debug, Deserialize, Serialize, JsonSchema, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct PrdCondition {
    /// Type of condition (e.g. `Ready`, `BriefingGenerated`).
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
    fn prd_spec_round_trips() {
        let spec = PrdSpec {
            repo: "5dlabs/sigma-1".to_string(),
            git_ref: "abc123".to_string(),
            prd_path: default_prd_path(),
            content_hash: "deadbeef".to_string(),
            content: None,
        };
        let json = serde_json::to_string(&spec).unwrap();
        assert!(json.contains("\"repo\":\"5dlabs/sigma-1\""));
        assert!(json.contains("\"ref\":\"abc123\""));
        assert!(json.contains("\"prdPath\":\"docs/prd.md\""));
        assert!(json.contains("\"contentHash\":\"deadbeef\""));
    }

    #[test]
    fn prd_status_defaults() {
        let s = PrdStatus::default();
        assert!(s.phase.is_none());
        assert!(s.conditions.is_none());
    }
}
