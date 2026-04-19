//! Morgan/OpenClaw hook forwarding helpers for reversible webhook cutover.

use anyhow::{anyhow, Context, Result};
use serde::Serialize;
use serde_json::Value;
use uuid::Uuid;

use crate::config::MorganDispatchConfig;

/// Verified webhook delivery metadata sent to Morgan.
#[derive(Debug, Clone)]
pub struct MorganWebhookDispatch {
    /// Upstream system name (e.g. `github`, `linear`, `gitlab`).
    pub source: &'static str,
    /// HTTP route that accepted the delivery.
    pub route: &'static str,
    /// Upstream event type or action when known.
    pub event_type: String,
    /// Provider delivery identifier if one exists.
    pub delivery_id: Option<String>,
    /// Whether PM already verified authenticity for this delivery.
    pub verified: bool,
    /// Extra labels to help Morgan reason about the event.
    pub labels: Vec<(&'static str, String)>,
    /// Parsed JSON payload.
    pub payload: Value,
}

/// Result returned after OpenClaw accepts a Morgan hook run.
#[derive(Debug, Clone)]
pub struct MorganDispatchAccepted {
    /// Target OpenClaw URL.
    pub hook_url: String,
    /// OpenClaw agent ID.
    pub agent_id: String,
    /// Session key used for the isolated run.
    pub session_key: String,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct OpenClawAgentHookPayload<'a> {
    message: String,
    name: String,
    agent_id: &'a str,
    session_key: String,
    wake_mode: &'static str,
    deliver: bool,
    timeout_seconds: u64,
}

/// Forward a verified webhook delivery to Morgan via OpenClaw hooks.
pub async fn dispatch_to_morgan(
    http_client: &reqwest::Client,
    config: &MorganDispatchConfig,
    dispatch: &MorganWebhookDispatch,
) -> Result<MorganDispatchAccepted> {
    let base_url = config.base_url.as_deref().ok_or_else(|| {
        anyhow!("MORGAN_HOOKS_BASE_URL is required when Morgan dispatch is enabled")
    })?;
    let token = config
        .token
        .as_deref()
        .ok_or_else(|| anyhow!("MORGAN_HOOKS_TOKEN is required when Morgan dispatch is enabled"))?;

    let hook_url = format!("{}/hooks/agent", base_url.trim_end_matches('/'));
    let session_key = build_session_key(
        &config.session_key_prefix,
        dispatch.source,
        &dispatch.event_type,
        dispatch.delivery_id.as_deref(),
    );

    let hook_payload = OpenClawAgentHookPayload {
        message: build_message(dispatch)?,
        name: format!("PM {}", dispatch.source.to_uppercase()),
        agent_id: &config.agent_id,
        session_key: session_key.clone(),
        wake_mode: "now",
        deliver: false,
        timeout_seconds: config.timeout_seconds,
    };

    let response = http_client
        .post(&hook_url)
        .bearer_auth(token)
        .json(&hook_payload)
        .send()
        .await
        .with_context(|| format!("Failed to POST webhook to Morgan at {hook_url}"))?;

    let status = response.status();
    if !status.is_success() {
        let body = response.text().await.unwrap_or_default();
        anyhow::bail!("Morgan hook returned {status}: {body}");
    }

    Ok(MorganDispatchAccepted {
        hook_url,
        agent_id: config.agent_id.clone(),
        session_key,
    })
}

fn build_message(dispatch: &MorganWebhookDispatch) -> Result<String> {
    let payload_json = serde_json::to_string_pretty(&dispatch.payload)
        .context("Failed to serialize webhook payload")?;

    let mut metadata_lines = vec![
        format!("- source: {}", dispatch.source),
        format!("- route: {}", dispatch.route),
        format!("- event_type: {}", dispatch.event_type),
        format!(
            "- delivery_id: {}",
            dispatch.delivery_id.as_deref().unwrap_or("unknown")
        ),
        format!("- verified_by_pm: {}", dispatch.verified),
    ];

    for (key, value) in &dispatch.labels {
        metadata_lines.push(format!("- {key}: {value}"));
    }

    Ok(format!(
        "A verified webhook arrived at the PM server.\n\n\
You are Morgan, the CTO orchestrator. Treat this as machine-triggered orchestration work. \
Decide whether to acknowledge only or to launch the appropriate workflow or agent \
(for example Stitch review, Atlas merge gating, play, intake, Lobster, or another CTO flow).\n\n\
Webhook metadata:\n{}\n\n\
Parsed payload:\n```json\n{}\n```",
        metadata_lines.join("\n"),
        payload_json
    ))
}

fn build_session_key(
    prefix: &str,
    source: &str,
    event_type: &str,
    delivery_id: Option<&str>,
) -> String {
    let cleaned_prefix = prefix.trim_end_matches(':');
    let event = sanitize_component(event_type);
    let source = sanitize_component(source);
    let delivery_seed = delivery_id.map_or_else(|| Uuid::new_v4().to_string(), ToOwned::to_owned);
    let delivery = sanitize_component(&delivery_seed);
    let mut session_key = format!("{cleaned_prefix}:{source}:{event}:{delivery}");

    if session_key.len() > 180 {
        session_key.truncate(180);
        while session_key.ends_with(':') || session_key.ends_with('-') {
            session_key.pop();
        }
    }

    session_key
}

fn sanitize_component(input: &str) -> String {
    let sanitized: String = input
        .chars()
        .map(|ch| {
            let lower = ch.to_ascii_lowercase();
            if lower.is_ascii_alphanumeric() {
                lower
            } else {
                '-'
            }
        })
        .collect();

    sanitized
        .split('-')
        .filter(|part| !part.is_empty())
        .collect::<Vec<_>>()
        .join("-")
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_build_message_includes_metadata_and_payload() {
        let dispatch = MorganWebhookDispatch {
            source: "github",
            route: "/webhooks/github/events",
            event_type: "pull_request".to_string(),
            delivery_id: Some("abc-123".to_string()),
            verified: true,
            labels: vec![("github_action", "opened".to_string())],
            payload: json!({"action": "opened", "number": 42}),
        };

        let message = build_message(&dispatch).unwrap();
        assert!(message.contains("verified webhook arrived"));
        assert!(message.contains("github_action: opened"));
        assert!(message.contains("\"number\": 42"));
    }

    #[test]
    fn test_build_session_key_uses_prefix_and_sanitized_components() {
        let key = build_session_key("hook:pm", "GitHub", "pull_request", Some("ABC_123"));
        assert_eq!(key, "hook:pm:github:pull-request:abc-123");
    }

    #[test]
    fn test_build_session_key_generates_uuid_when_missing_delivery() {
        let key = build_session_key("hook:pm", "linear", "AgentSessionEvent", None);
        assert!(key.starts_with("hook:pm:linear:agentsessionevent:"));
        assert!(key.len() > "hook:pm:linear:agentsessionevent:".len());
    }
}
