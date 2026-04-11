//! Escalation policy and per-agent session state.
//!
//! This module is intentionally free of I/O and server plumbing so the decision
//! logic can be unit-tested in isolation. The HTTP server consumes these types
//! from `BridgeState` and wires them into the `tools/list` / `tools/call`
//! handlers.
//!
//! See `docs/architecture/dynamic-mcp-tools.md` (added in PR 2) for the full
//! design rationale.

use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};

/// How the server should respond to `tools_request_capability` calls.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EscalationMode {
    /// Grant any tool present in the catalog. `deny` globs still apply.
    Auto,
    /// Grant only tools matching at least one `allow` glob. `deny` globs still apply.
    #[default]
    Allowlist,
    /// Deny every escalation and record it for human review.
    Review,
}

/// Policy applied to escalation requests.
///
/// Stored in `BridgeState` as a global default in PR 1. PR 2 will thread a
/// per-session policy through CRD → controller → `X-Agent-Policy` header.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct EscalationPolicy {
    #[serde(default)]
    pub mode: EscalationMode,
    /// Glob patterns allowing a tool. Only consulted when `mode == Allowlist`.
    #[serde(default)]
    pub allow: Vec<String>,
    /// Glob patterns blocking a tool regardless of `mode`. Takes precedence over `allow`.
    #[serde(default)]
    pub deny: Vec<String>,
}

/// Outcome of evaluating a single escalation request.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EscalationDecision {
    Grant,
    Deny { reason: String },
}

impl EscalationDecision {
    pub fn is_grant(&self) -> bool {
        matches!(self, Self::Grant)
    }

    pub fn label(&self) -> &'static str {
        match self {
            Self::Grant => "grant",
            Self::Deny { .. } => "deny",
        }
    }
}

/// Audit entry recorded in session state every time an escalation is evaluated.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EscalationRecord {
    pub tool_name: String,
    pub reason: String,
    pub decision: String,
    /// Populated on deny with the human-readable policy reason.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub policy_reason: Option<String>,
    /// RFC 3339 timestamp.
    pub at: String,
}

/// Per-agent ephemeral state held by the HTTP server.
///
/// Keyed in `BridgeState` by the `X-Agent-Id` header. Cleared when the session
/// TTL expires (PR 2 adds the eviction loop; PR 1 leaves entries in place).
#[derive(Debug, Clone, Default)]
pub struct SessionState {
    /// Tools pre-warmed at session start. These are always eager and never deferred.
    pub prewarm: HashSet<String>,
    /// Tools granted via escalation during this session.
    pub granted: HashSet<String>,
    /// Full audit log of escalations (grants + denies) for this session.
    pub escalations: Vec<EscalationRecord>,
}

impl SessionState {
    pub fn new(prewarm: HashSet<String>) -> Self {
        Self {
            prewarm,
            granted: HashSet::new(),
            escalations: Vec::new(),
        }
    }

    /// True when the tool is already accessible to this session (no escalation needed).
    pub fn has(&self, tool: &str) -> bool {
        self.prewarm.contains(tool) || self.granted.contains(tool)
    }
}

/// Map of agent id → session state. Held inside `BridgeState` behind a lock.
pub type SessionMap = HashMap<String, SessionState>;

/// Evaluate a single escalation request without touching any session state.
///
/// `catalog_has` lets the caller plug in whatever resolution logic they prefer
/// (exact match, fuzzy match, etc.). Keeping it as a closure keeps this module
/// pure and trivially testable.
pub fn evaluate(
    policy: &EscalationPolicy,
    tool: &str,
    catalog_has: impl Fn(&str) -> bool,
) -> EscalationDecision {
    if tool.is_empty() {
        return EscalationDecision::Deny {
            reason: "empty tool name".into(),
        };
    }
    if !catalog_has(tool) {
        return EscalationDecision::Deny {
            reason: format!("'{tool}' is not in the tool catalog"),
        };
    }
    if policy.deny.iter().any(|pat| glob_match(pat, tool)) {
        return EscalationDecision::Deny {
            reason: format!("'{tool}' matches a deny pattern"),
        };
    }
    match policy.mode {
        EscalationMode::Auto => EscalationDecision::Grant,
        EscalationMode::Allowlist => {
            if policy.allow.iter().any(|pat| glob_match(pat, tool)) {
                EscalationDecision::Grant
            } else {
                EscalationDecision::Deny {
                    reason: format!("'{tool}' matches no allow pattern"),
                }
            }
        }
        EscalationMode::Review => EscalationDecision::Deny {
            reason: "policy mode is 'review': request logged for human approval".into(),
        },
    }
}

/// Minimal glob matcher supporting only `*` (any run of characters).
///
/// We intentionally avoid pulling in `globset` — the tool-name match surface is
/// tiny and the patterns we expect to see (`github_*`, `grafana_query_*`, etc.)
/// don't need character classes or `?`.
pub fn glob_match(pattern: &str, text: &str) -> bool {
    let parts: Vec<&str> = pattern.split('*').collect();

    // No wildcards → literal compare.
    if parts.len() == 1 {
        return parts[0] == text;
    }

    let mut rest = text;

    // First segment: must match at the start unless the pattern began with '*'.
    if let Some(first) = parts.first() {
        if !first.is_empty() {
            if !rest.starts_with(first) {
                return false;
            }
            rest = &rest[first.len()..];
        }
    }

    // Middle segments: must appear in order, leaving some suffix for the last segment.
    let last_idx = parts.len() - 1;
    for mid in &parts[1..last_idx] {
        if mid.is_empty() {
            continue; // `**` collapses to `*`
        }
        match rest.find(mid) {
            Some(i) => rest = &rest[i + mid.len()..],
            None => return false,
        }
    }

    // Last segment: must match at the end unless the pattern ended with '*'.
    let last = parts[last_idx];
    if last.is_empty() {
        true
    } else {
        rest.ends_with(last) && rest.len() >= last.len()
    }
}

/// Parse a whitespace-separated `X-Agent-Prewarm` header into a set of tool names.
pub fn parse_prewarm_header(raw: &str) -> HashSet<String> {
    raw.split_whitespace().map(str::to_string).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn catalog<'a>(tools: &'a [&'static str]) -> impl Fn(&str) -> bool + 'a {
        move |name: &str| tools.contains(&name)
    }

    // ---- glob_match ----------------------------------------------------------

    #[test]
    fn glob_exact_match() {
        assert!(glob_match("github_search_code", "github_search_code"));
        assert!(!glob_match("github_search_code", "github_search_issues"));
    }

    #[test]
    fn glob_leading_wildcard() {
        assert!(glob_match("*_search_code", "github_search_code"));
        assert!(glob_match("*_search_code", "octocode_search_code"));
        assert!(!glob_match("*_search_code", "github_search_issues"));
    }

    #[test]
    fn glob_trailing_wildcard() {
        assert!(glob_match("github_*", "github_search_code"));
        assert!(glob_match("github_*", "github_"));
        assert!(!glob_match("github_*", "grafana_query"));
    }

    #[test]
    fn glob_infix_wildcard() {
        assert!(glob_match("grafana_*_logs", "grafana_query_loki_logs"));
        assert!(!glob_match("grafana_*_logs", "grafana_query_prometheus"));
    }

    #[test]
    fn glob_multiple_wildcards() {
        assert!(glob_match("*_*_code", "github_search_code"));
        assert!(!glob_match("*_*_code", "github_code")); // not enough underscores
    }

    #[test]
    fn glob_only_wildcard_matches_all() {
        assert!(glob_match("*", ""));
        assert!(glob_match("*", "anything"));
    }

    #[test]
    fn glob_empty_pattern_only_matches_empty() {
        assert!(glob_match("", ""));
        assert!(!glob_match("", "nonempty"));
    }

    #[test]
    fn glob_collapsed_wildcards() {
        assert!(glob_match("github_**_code", "github_search_code"));
    }

    // ---- evaluate ------------------------------------------------------------

    #[test]
    fn evaluate_empty_tool_denies() {
        let policy = EscalationPolicy::default();
        let d = evaluate(&policy, "", |_| true);
        assert!(matches!(d, EscalationDecision::Deny { .. }));
    }

    #[test]
    fn evaluate_tool_not_in_catalog_denies() {
        let policy = EscalationPolicy {
            mode: EscalationMode::Auto,
            ..Default::default()
        };
        let d = evaluate(&policy, "nope", catalog(&["github_search_code"]));
        match d {
            EscalationDecision::Deny { reason } => {
                assert!(reason.contains("not in the tool catalog"))
            }
            EscalationDecision::Grant => panic!("expected deny"),
        }
    }

    #[test]
    fn evaluate_auto_grants_anything_in_catalog() {
        let policy = EscalationPolicy {
            mode: EscalationMode::Auto,
            ..Default::default()
        };
        assert_eq!(
            evaluate(
                &policy,
                "github_search_code",
                catalog(&["github_search_code"])
            ),
            EscalationDecision::Grant
        );
    }

    #[test]
    fn evaluate_auto_still_honors_deny() {
        let policy = EscalationPolicy {
            mode: EscalationMode::Auto,
            allow: vec![],
            deny: vec!["*_delete_*".into()],
        };
        let d = evaluate(
            &policy,
            "github_delete_file",
            catalog(&["github_delete_file"]),
        );
        assert!(matches!(d, EscalationDecision::Deny { .. }));
    }

    #[test]
    fn evaluate_allowlist_matches_allow() {
        let policy = EscalationPolicy {
            mode: EscalationMode::Allowlist,
            allow: vec!["github_*".into()],
            deny: vec![],
        };
        assert_eq!(
            evaluate(
                &policy,
                "github_search_code",
                catalog(&["github_search_code"])
            ),
            EscalationDecision::Grant
        );
    }

    #[test]
    fn evaluate_allowlist_no_match_denies() {
        let policy = EscalationPolicy {
            mode: EscalationMode::Allowlist,
            allow: vec!["github_*".into()],
            deny: vec![],
        };
        let d = evaluate(
            &policy,
            "grafana_query_loki",
            catalog(&["grafana_query_loki"]),
        );
        match d {
            EscalationDecision::Deny { reason } => assert!(reason.contains("no allow pattern")),
            EscalationDecision::Grant => panic!("expected deny"),
        }
    }

    #[test]
    fn evaluate_allowlist_deny_beats_allow() {
        let policy = EscalationPolicy {
            mode: EscalationMode::Allowlist,
            allow: vec!["github_*".into()],
            deny: vec!["github_delete_*".into()],
        };
        let d = evaluate(
            &policy,
            "github_delete_file",
            catalog(&["github_delete_file"]),
        );
        assert!(matches!(d, EscalationDecision::Deny { .. }));
    }

    #[test]
    fn evaluate_review_denies_everything() {
        let policy = EscalationPolicy {
            mode: EscalationMode::Review,
            allow: vec!["*".into()],
            deny: vec![],
        };
        let d = evaluate(
            &policy,
            "github_search_code",
            catalog(&["github_search_code"]),
        );
        match d {
            EscalationDecision::Deny { reason } => assert!(reason.contains("review")),
            EscalationDecision::Grant => panic!("expected deny in review mode"),
        }
    }

    // ---- SessionState --------------------------------------------------------

    #[test]
    fn session_has_checks_prewarm_and_granted() {
        let mut s = SessionState::new(["github_search_code".into()].into_iter().collect());
        assert!(s.has("github_search_code"));
        assert!(!s.has("grafana_query_loki"));

        s.granted.insert("grafana_query_loki".into());
        assert!(s.has("grafana_query_loki"));
    }

    #[test]
    fn session_default_is_empty() {
        let s = SessionState::default();
        assert!(!s.has("github_search_code"));
        assert!(s.prewarm.is_empty());
        assert!(s.granted.is_empty());
    }

    // ---- parse_prewarm_header ------------------------------------------------

    #[test]
    fn parse_prewarm_splits_on_any_whitespace() {
        let set = parse_prewarm_header(
            "github_search_code  grafana_query_loki\tterraform_search_modules",
        );
        assert_eq!(set.len(), 3);
        assert!(set.contains("github_search_code"));
        assert!(set.contains("grafana_query_loki"));
        assert!(set.contains("terraform_search_modules"));
    }

    #[test]
    fn parse_prewarm_empty_is_empty_set() {
        assert!(parse_prewarm_header("").is_empty());
        assert!(parse_prewarm_header("   \t\n ").is_empty());
    }

    // ---- EscalationDecision helpers -----------------------------------------

    #[test]
    fn decision_is_grant() {
        assert!(EscalationDecision::Grant.is_grant());
        assert!(!EscalationDecision::Deny { reason: "x".into() }.is_grant());
    }

    #[test]
    fn decision_label() {
        assert_eq!(EscalationDecision::Grant.label(), "grant");
        assert_eq!(
            EscalationDecision::Deny { reason: "x".into() }.label(),
            "deny"
        );
    }

    // ---- EscalationMode default ---------------------------------------------

    #[test]
    fn mode_default_is_allowlist() {
        assert_eq!(EscalationMode::default(), EscalationMode::Allowlist);
    }
}
