//! GitHub agent interaction handlers.
//!
//! Handles:
//! - @mentions in PR comments (triggers agent actions)
//! - Remediation button clicks (triggers agent fixes)

use axum::{
    body::Bytes,
    extract::State,
    http::{HeaderMap, StatusCode},
    response::Json,
};
use regex::Regex;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::sync::Arc;
use tracing::{debug, error, info, warn};

use super::callbacks::CallbackState;
use crate::config::WebhookDispatchMode;
use crate::morgan_hooks::{dispatch_to_morgan, MorganWebhookDispatch};

// =============================================================================
// Types
// =============================================================================

/// Supported agents for @mentions and remediation
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Agent {
    // Implementation agents
    Rex,
    Grizz,
    Nova,
    Blaze,
    Tap,
    Spark,
    Angie,
    Vex,
    Forge,
    // Support agents
    Stitch,
    Cleo,
    Cipher,
    Tess,
}

impl Agent {
    /// Get the GitHub App name for this agent
    #[must_use]
    pub fn github_app_name(&self) -> &'static str {
        match self {
            Self::Rex => "5DLabs-Rex",
            Self::Grizz => "5DLabs-Grizz",
            Self::Nova => "5DLabs-Nova",
            Self::Blaze => "5DLabs-Blaze",
            Self::Tap => "5DLabs-Tap",
            Self::Spark => "5DLabs-Spark",
            Self::Angie => "5DLabs-Angie",
            Self::Vex => "5DLabs-Vex",
            Self::Forge => "5DLabs-Forge",
            Self::Stitch => "5DLabs-Stitch",
            Self::Cleo => "5DLabs-Cleo",
            Self::Cipher => "5DLabs-Cipher",
            Self::Tess => "5DLabs-Tess",
        }
    }

    /// Get the agent name as lowercase string
    #[must_use]
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Rex => "rex",
            Self::Grizz => "grizz",
            Self::Nova => "nova",
            Self::Blaze => "blaze",
            Self::Tap => "tap",
            Self::Spark => "spark",
            Self::Angie => "angie",
            Self::Vex => "vex",
            Self::Forge => "forge",
            Self::Stitch => "stitch",
            Self::Cleo => "cleo",
            Self::Cipher => "cipher",
            Self::Tess => "tess",
        }
    }

    /// Parse agent from @mention string (case insensitive)
    #[must_use]
    pub fn from_mention(mention: &str) -> Option<Self> {
        let lower = mention.to_lowercase();
        match lower.as_str() {
            "rex" | "5dlabs-rex" | "@5dlabs-rex" => Some(Self::Rex),
            "grizz" | "5dlabs-grizz" | "@5dlabs-grizz" => Some(Self::Grizz),
            "nova" | "5dlabs-nova" | "@5dlabs-nova" => Some(Self::Nova),
            "blaze" | "5dlabs-blaze" | "@5dlabs-blaze" => Some(Self::Blaze),
            "tap" | "5dlabs-tap" | "@5dlabs-tap" => Some(Self::Tap),
            "spark" | "5dlabs-spark" | "@5dlabs-spark" => Some(Self::Spark),
            "angie" | "5dlabs-angie" | "@5dlabs-angie" => Some(Self::Angie),
            "vex" | "5dlabs-vex" | "@5dlabs-vex" => Some(Self::Vex),
            "forge" | "5dlabs-forge" | "@5dlabs-forge" => Some(Self::Forge),
            "stitch" | "5dlabs-stitch" | "@5dlabs-stitch" => Some(Self::Stitch),
            "cleo" | "5dlabs-cleo" | "@5dlabs-cleo" => Some(Self::Cleo),
            "cipher" | "5dlabs-cipher" | "@5dlabs-cipher" => Some(Self::Cipher),
            "tess" | "5dlabs-tess" | "@5dlabs-tess" => Some(Self::Tess),
            _ => None,
        }
    }

    /// Get the default run type for this agent
    #[must_use]
    pub fn default_run_type(&self) -> &'static str {
        match self {
            Self::Stitch | Self::Cleo => "review",
            Self::Cipher => "security",
            Self::Tess => "test",
            _ => "remediation",
        }
    }

    /// Get the primary language/stack this agent handles
    #[must_use]
    pub fn primary_language(&self) -> Option<&'static str> {
        match self {
            Self::Rex => Some("rust"),
            Self::Grizz => Some("go"),
            Self::Nova => Some("typescript"),
            Self::Blaze => Some("react"),
            Self::Tap => Some("react-native"),
            Self::Spark => Some("electron"),
            Self::Angie => Some("agentic"),
            Self::Vex => Some("unity"),
            Self::Forge => Some("unreal"),
            _ => None,
        }
    }
}

/// Language detected from file extensions
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Language {
    Rust,
    Go,
    TypeScript,
    JavaScript,
    Python,
    CSharp,
    Cpp,
    React,
    ReactNative,
    Unknown,
}

impl Language {
    /// Detect language from file path
    #[must_use]
    #[allow(clippy::case_sensitive_file_extension_comparisons)]
    pub fn from_path(path: &str) -> Self {
        let lower = path.to_lowercase();

        // Check for React/React Native patterns first
        if lower.contains("/components/")
            || lower.contains("/pages/")
            || lower.contains("/app/")
            || lower.ends_with(".tsx")
            || lower.ends_with(".jsx")
        {
            if lower.contains("native") || lower.contains("/ios/") || lower.contains("/android/") {
                return Self::ReactNative;
            }
            return Self::React;
        }

        // Check by extension
        if lower.ends_with(".rs") {
            Self::Rust
        } else if lower.ends_with(".go") {
            Self::Go
        } else if lower.ends_with(".ts") || lower.ends_with(".tsx") {
            Self::TypeScript
        } else if lower.ends_with(".js") || lower.ends_with(".jsx") {
            Self::JavaScript
        } else if lower.ends_with(".py") {
            Self::Python
        } else if lower.ends_with(".cs") {
            Self::CSharp
        } else if lower.ends_with(".cpp")
            || lower.ends_with(".cc")
            || lower.ends_with(".cxx")
            || lower.ends_with(".h")
            || lower.ends_with(".hpp")
        {
            Self::Cpp
        } else {
            Self::Unknown
        }
    }

    /// Get the recommended agent for this language
    #[must_use]
    pub fn recommended_agent(&self) -> Agent {
        match self {
            Self::Rust => Agent::Rex,
            Self::Go => Agent::Grizz,
            Self::TypeScript | Self::JavaScript | Self::React => Agent::Blaze,
            Self::ReactNative => Agent::Tap,
            Self::CSharp => Agent::Vex,
            Self::Cpp => Agent::Forge,
            Self::Python => Agent::Nova, // Default to Nova for Python
            Self::Unknown => Agent::Rex, // Default to Rex for unknown
        }
    }
}

/// Parsed @mention from a comment
#[derive(Debug, Clone)]
pub struct ParsedMention {
    /// The agent being mentioned
    pub agent: Agent,
    /// Instructions following the mention
    pub instructions: String,
    /// Full original comment text
    pub full_comment: String,
}

/// PR context from webhook payload
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrContext {
    /// PR number
    pub number: u64,
    /// PR title
    pub title: String,
    /// Repository full name (org/repo)
    pub repo_full_name: String,
    /// Repository clone URL
    pub clone_url: String,
    /// PR head branch
    pub head_branch: String,
    /// PR head SHA
    pub head_sha: String,
    /// PR base branch
    pub base_branch: String,
    /// PR HTML URL
    pub html_url: String,
}

/// GitHub issue comment payload
#[derive(Debug, Clone, Deserialize)]
pub struct IssueCommentPayload {
    pub action: String,
    pub issue: Issue,
    pub comment: Comment,
    pub repository: Repository,
}

/// GitHub pull request review comment payload
#[derive(Debug, Clone, Deserialize)]
pub struct ReviewCommentPayload {
    pub action: String,
    pub pull_request: PullRequest,
    pub comment: ReviewComment,
    pub repository: Repository,
}

/// GitHub `check_run` payload for button clicks
#[derive(Debug, Clone, Deserialize)]
pub struct CheckRunPayload {
    pub action: String,
    pub check_run: CheckRun,
    pub requested_action: Option<RequestedAction>,
    pub repository: Repository,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Issue {
    pub number: u64,
    pub title: String,
    pub pull_request: Option<PullRequestRef>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct PullRequestRef {
    pub url: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct PullRequest {
    pub number: u64,
    pub title: String,
    pub head: GitRef,
    pub base: GitRef,
    pub html_url: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct GitRef {
    #[serde(rename = "ref")]
    pub ref_name: String,
    pub sha: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Comment {
    pub id: u64,
    pub body: String,
    pub user: User,
    pub html_url: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ReviewComment {
    pub id: u64,
    pub body: String,
    pub user: User,
    pub path: String,
    pub position: Option<u64>,
    pub html_url: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct User {
    pub login: String,
    #[serde(rename = "type")]
    pub user_type: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Repository {
    pub id: u64,
    pub name: String,
    pub full_name: String,
    pub clone_url: String,
    pub html_url: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct CheckRun {
    pub id: u64,
    pub name: String,
    pub head_sha: String,
    pub status: String,
    pub conclusion: Option<String>,
    pub pull_requests: Vec<CheckRunPr>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct CheckRunPr {
    pub number: u64,
    pub head: GitRef,
    pub base: GitRef,
}

#[derive(Debug, Clone, Deserialize)]
pub struct RequestedAction {
    pub identifier: String,
}

// =============================================================================
// Mention Parsing
// =============================================================================

/// Parse @mentions from a comment body
///
/// # Panics
///
/// Panics if the regex pattern is invalid (this is a compile-time constant).
#[must_use]
pub fn parse_mentions(comment: &str) -> Vec<ParsedMention> {
    let re = Regex::new(
        r"(?i)@5dlabs-(stitch|rex|grizz|nova|blaze|tap|spark|angie|vex|forge|cleo|cipher|tess)\s*(.*)",
    )
    .unwrap();

    let mut mentions = Vec::new();

    for cap in re.captures_iter(comment) {
        let agent_name = cap.get(1).map_or("", |m| m.as_str());
        let instructions = cap.get(2).map_or("", |m| m.as_str()).trim().to_string();

        if let Some(agent) = Agent::from_mention(agent_name) {
            mentions.push(ParsedMention {
                agent,
                instructions,
                full_comment: comment.to_string(),
            });
        }
    }

    mentions
}

/// Parse remediation button identifier
/// Format: fix-<agent>-pr<number>-<check_run_id>
///
/// # Panics
///
/// Panics if the regex pattern is invalid (this is a compile-time constant).
#[must_use]
pub fn parse_button_identifier(identifier: &str) -> Option<(Agent, u64, u64)> {
    let re = Regex::new(r"^fix-(rex|grizz|nova|blaze|tap|spark|angie|vex|forge)-pr(\d+)-(\d+)$")
        .unwrap();

    if let Some(cap) = re.captures(identifier) {
        let agent_name = cap.get(1)?.as_str();
        let pr_number: u64 = cap.get(2)?.as_str().parse().ok()?;
        let check_run_id: u64 = cap.get(3)?.as_str().parse().ok()?;

        let agent = Agent::from_mention(agent_name)?;
        return Some((agent, pr_number, check_run_id));
    }

    None
}

async fn maybe_dispatch_legacy_agent_webhook_to_morgan(
    state: &CallbackState,
    route: &'static str,
    event_type: &'static str,
    payload: &Value,
) -> Result<Option<Json<Value>>, StatusCode> {
    let dispatch_mode = state.morgan_dispatch.mode;
    if !dispatch_mode.dispatches_to_morgan() {
        return Ok(None);
    }

    let dispatch = MorganWebhookDispatch {
        source: "github",
        route,
        event_type: event_type.to_string(),
        delivery_id: payload
            .pointer("/check_run/id")
            .and_then(Value::as_u64)
            .map(|id| id.to_string()),
        verified: false,
        labels: vec![("legacy_route", route.to_string())],
        payload: payload.clone(),
    };

    match dispatch_to_morgan(&state.http_client, &state.morgan_dispatch, &dispatch).await {
        Ok(accepted) => {
            info!(
                session_key = %accepted.session_key,
                agent_id = %accepted.agent_id,
                route,
                "Forwarded legacy agent webhook to Morgan"
            );

            if dispatch_mode == WebhookDispatchMode::Morgan {
                return Ok(Some(Json(json!({
                    "status": "accepted",
                    "dispatch": "morgan",
                    "source": "github",
                    "event_type": event_type,
                    "route": route,
                    "session_key": accepted.session_key,
                    "agent_id": accepted.agent_id
                }))));
            }

            Ok(None)
        }
        Err(e) => {
            if dispatch_mode == WebhookDispatchMode::Shadow {
                warn!(error = %e, route, "Failed to shadow-dispatch legacy agent webhook to Morgan");
                Ok(None)
            } else {
                error!(error = %e, route, "Failed to dispatch legacy agent webhook to Morgan");
                Err(StatusCode::BAD_GATEWAY)
            }
        }
    }
}

// =============================================================================
// Handlers
// =============================================================================

/// Handle @mention webhook from Argo Events sensor
#[allow(clippy::too_many_lines)]
pub async fn handle_mention_webhook(
    State(state): State<Arc<CallbackState>>,
    _headers: HeaderMap,
    body: Bytes,
) -> Result<Json<Value>, StatusCode> {
    info!("Received @mention webhook");

    // Parse the payload
    let payload: Value = serde_json::from_slice(&body).map_err(|e| {
        error!("Failed to parse mention payload: {}", e);
        StatusCode::BAD_REQUEST
    })?;

    // Extract nested payload (from Argo Events sensor)
    let inner_payload = payload.get("payload").unwrap_or(&payload);

    if let Some(response) = maybe_dispatch_legacy_agent_webhook_to_morgan(
        &state,
        "/webhooks/github/mention",
        "mention",
        inner_payload,
    )
    .await?
    {
        return Ok(response);
    }

    handle_mention_webhook_impl(&state, inner_payload).await
}

/// Inner implementation for mention webhook (callable from unified handler).
///
/// Accepts raw GitHub event bytes (no Argo Events wrapping).
pub async fn handle_mention_webhook_inner(
    state: &CallbackState,
    body: &[u8],
) -> Result<Json<Value>, StatusCode> {
    let payload: Value = serde_json::from_slice(body).map_err(|e| {
        error!("Failed to parse mention payload: {}", e);
        StatusCode::BAD_REQUEST
    })?;
    handle_mention_webhook_impl(state, &payload).await
}

/// Shared implementation for mention webhook processing.
/// Detect the mention source event type from a GitHub webhook payload.
fn detect_mention_source(payload: &Value) -> String {
    payload
        .get("X-GitHub-Event")
        .and_then(|v| v.as_str())
        .map_or_else(
            || {
                if payload.get("issue").is_some() && payload.get("comment").is_some() {
                    "issue_comment".to_string()
                } else if payload.get("pull_request").is_some() && payload.get("comment").is_some()
                {
                    "pull_request_review_comment".to_string()
                } else {
                    "unknown".to_string()
                }
            },
            String::from,
        )
}

/// Extract PR context, comment body, and comment URL from a mention webhook payload.
///
/// Returns `None` if the comment is on an issue (not a PR).
fn extract_mention_context(
    source: &str,
    payload: &Value,
) -> Result<Option<(PrContext, String, String)>, StatusCode> {
    match source {
        "issue_comment" => {
            let event: IssueCommentPayload =
                serde_json::from_value(payload.clone()).map_err(|e| {
                    error!("Failed to parse issue_comment payload: {}", e);
                    StatusCode::BAD_REQUEST
                })?;

            if event.issue.pull_request.is_none() {
                debug!("Comment is on issue, not PR - ignoring");
                return Ok(None);
            }

            let pr_context = PrContext {
                number: event.issue.number,
                title: event.issue.title.clone(),
                repo_full_name: event.repository.full_name.clone(),
                clone_url: event.repository.clone_url.clone(),
                head_branch: String::new(),
                head_sha: String::new(),
                base_branch: String::new(),
                html_url: format!("{}/pull/{}", event.repository.html_url, event.issue.number),
            };

            Ok(Some((
                pr_context,
                event.comment.body,
                event.comment.html_url,
            )))
        }
        "pull_request_review_comment" => {
            let event: ReviewCommentPayload =
                serde_json::from_value(payload.clone()).map_err(|e| {
                    error!("Failed to parse review_comment payload: {}", e);
                    StatusCode::BAD_REQUEST
                })?;

            let pr_context = PrContext {
                number: event.pull_request.number,
                title: event.pull_request.title.clone(),
                repo_full_name: event.repository.full_name.clone(),
                clone_url: event.repository.clone_url.clone(),
                head_branch: event.pull_request.head.ref_name.clone(),
                head_sha: event.pull_request.head.sha.clone(),
                base_branch: event.pull_request.base.ref_name.clone(),
                html_url: event.pull_request.html_url.clone(),
            };

            Ok(Some((
                pr_context,
                event.comment.body,
                event.comment.html_url,
            )))
        }
        _ => {
            warn!(source = %source, "Unknown mention source");
            Err(StatusCode::BAD_REQUEST)
        }
    }
}

async fn handle_mention_webhook_impl(
    state: &CallbackState,
    inner_payload: &Value,
) -> Result<Json<Value>, StatusCode> {
    let mention_source = detect_mention_source(inner_payload);
    info!(source = %mention_source, "Detected mention source");

    let Some((pr_context, comment_body, comment_url)) =
        extract_mention_context(&mention_source, inner_payload)?
    else {
        return Ok(Json(json!({
            "status": "ignored",
            "reason": "not a PR comment"
        })));
    };

    // Parse mentions from comment
    let mentions = parse_mentions(&comment_body);
    if mentions.is_empty() {
        debug!("No valid @mentions found in comment");
        return Ok(Json(json!({
            "status": "ignored",
            "reason": "no valid mentions"
        })));
    }

    // Process each mention (usually just one)
    let mut created_runs = Vec::new();
    for mention in mentions {
        info!(
            agent = %mention.agent.as_str(),
            pr = %pr_context.number,
            repo = %pr_context.repo_full_name,
            instructions = %mention.instructions,
            "Processing @mention"
        );

        // Create CodeRun CR for this mention
        match create_mention_coderun(state, &mention, &pr_context, &comment_url).await {
            Ok(run_name) => {
                info!(run_name = %run_name, "Created CodeRun for mention");
                created_runs.push(run_name);
            }
            Err(e) => {
                error!(error = %e, "Failed to create CodeRun for mention");
            }
        }
    }

    Ok(Json(json!({
        "status": "ok",
        "created_runs": created_runs,
        "pr": pr_context.number,
        "repo": pr_context.repo_full_name
    })))
}

/// Handle remediation button click webhook (Axum handler).
pub async fn handle_remediation_webhook(
    State(state): State<Arc<CallbackState>>,
    _headers: HeaderMap,
    body: Bytes,
) -> Result<Json<Value>, StatusCode> {
    info!("Received remediation button webhook");

    // Parse the payload
    let payload: Value = serde_json::from_slice(&body).map_err(|e| {
        error!("Failed to parse remediation payload: {}", e);
        StatusCode::BAD_REQUEST
    })?;

    // Extract nested payload (from Argo Events sensor)
    let inner_payload = payload.get("payload").unwrap_or(&payload);

    if let Some(response) = maybe_dispatch_legacy_agent_webhook_to_morgan(
        &state,
        "/webhooks/github/remediation",
        "remediation",
        inner_payload,
    )
    .await?
    {
        return Ok(response);
    }

    handle_remediation_webhook_impl(&state, inner_payload).await
}

/// Inner implementation for remediation webhook (callable from unified handler).
///
/// Accepts raw GitHub event bytes (no Argo Events wrapping).
pub async fn handle_remediation_webhook_inner(
    state: &CallbackState,
    body: &[u8],
) -> Result<Json<Value>, StatusCode> {
    let payload: Value = serde_json::from_slice(body).map_err(|e| {
        error!("Failed to parse remediation payload: {}", e);
        StatusCode::BAD_REQUEST
    })?;
    handle_remediation_webhook_impl(state, &payload).await
}

/// Shared implementation for remediation webhook processing.
async fn handle_remediation_webhook_impl(
    state: &CallbackState,
    inner_payload: &Value,
) -> Result<Json<Value>, StatusCode> {
    let event: CheckRunPayload = serde_json::from_value(inner_payload.clone()).map_err(|e| {
        error!("Failed to parse check_run payload: {}", e);
        StatusCode::BAD_REQUEST
    })?;

    // Must have requested_action
    let requested_action = event.requested_action.as_ref().ok_or_else(|| {
        warn!("No requested_action in check_run event");
        StatusCode::BAD_REQUEST
    })?;

    // Parse the button identifier
    let (agent, pr_number, check_run_id) = parse_button_identifier(&requested_action.identifier)
        .ok_or_else(|| {
            warn!(
                identifier = %requested_action.identifier,
                "Invalid button identifier format"
            );
            StatusCode::BAD_REQUEST
        })?;

    info!(
        agent = %agent.as_str(),
        pr_number = %pr_number,
        check_run_id = %check_run_id,
        repo = %event.repository.full_name,
        "Processing remediation button click"
    );

    // Get PR context from check_run
    let pr = event.check_run.pull_requests.first().ok_or_else(|| {
        warn!("No PR associated with check_run");
        StatusCode::BAD_REQUEST
    })?;

    let pr_context = PrContext {
        number: pr_number,
        title: format!("PR #{pr_number}"), // We don't have title in check_run payload
        repo_full_name: event.repository.full_name.clone(),
        clone_url: event.repository.clone_url.clone(),
        head_branch: pr.head.ref_name.clone(),
        head_sha: pr.head.sha.clone(),
        base_branch: pr.base.ref_name.clone(),
        html_url: format!("{}/pull/{}", event.repository.html_url, pr_number),
    };

    // Create CodeRun for remediation
    let run_name = create_remediation_coderun(
        state,
        agent,
        &pr_context,
        &event.check_run.name,
        check_run_id,
    )
    .await
    .map_err(|e| {
        error!(error = %e, "Failed to create remediation CodeRun");
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    info!(run_name = %run_name, "Created CodeRun for remediation");

    Ok(Json(json!({
        "status": "ok",
        "run_name": run_name,
        "agent": agent.as_str(),
        "pr": pr_number,
        "repo": event.repository.full_name
    })))
}

// =============================================================================
// CodeRun Creation
// =============================================================================

/// Create a `CodeRun` CR for an @mention
async fn create_mention_coderun(
    _state: &CallbackState,
    mention: &ParsedMention,
    pr_context: &PrContext,
    comment_url: &str,
) -> Result<String, String> {
    use kube::{api::PostParams, Api};

    let run_name = format!(
        "mention-{}-pr{}-{}",
        mention.agent.as_str(),
        pr_context.number,
        chrono::Utc::now().timestamp()
    );

    let coderun = serde_json::json!({
        "apiVersion": "agents.platform/v1",
        "kind": "CodeRun",
        "metadata": {
            "name": run_name,
            "namespace": "cto",
            "labels": {
                "trigger": "mention",
                "agent": mention.agent.as_str(),
                "pr-number": pr_context.number.to_string()
            }
        },
        "spec": {
            "runType": mention.agent.default_run_type(),
            "service": pr_context.repo_full_name.split('/').next_back().unwrap_or("unknown"),
            "repositoryUrl": pr_context.clone_url,
            "docsRepositoryUrl": pr_context.clone_url,
            "docsProjectDirectory": ".",
            "workingDirectory": ".",
            "githubApp": mention.agent.github_app_name(),
            "model": "claude-opus-4-5-20251101",
            "env": {
                "PR_NUMBER": pr_context.number.to_string(),
                "PR_BRANCH": pr_context.head_branch,
                "PR_SHA": pr_context.head_sha,
                "PR_URL": pr_context.html_url,
                "REPO_FULL_NAME": pr_context.repo_full_name,
                "TRIGGER": "mention",
                "COMMENT_URL": comment_url,
                "INSTRUCTIONS": mention.instructions
            }
        }
    });

    // Apply the CR via kubectl (we have cluster access)
    let client = kube::Client::try_default()
        .await
        .map_err(|e| format!("Failed to create k8s client: {e}"))?;

    let api: Api<kube::api::DynamicObject> = Api::namespaced_with(
        client,
        "cto",
        &kube::api::ApiResource {
            group: "agents.platform".to_string(),
            version: "v1".to_string(),
            kind: "CodeRun".to_string(),
            api_version: "agents.platform/v1".to_string(),
            plural: "coderuns".to_string(),
        },
    );

    let obj: kube::api::DynamicObject =
        serde_json::from_value(coderun).map_err(|e| format!("Failed to serialize CodeRun: {e}"))?;

    api.create(&PostParams::default(), &obj)
        .await
        .map_err(|e| format!("Failed to create CodeRun: {e}"))?;

    Ok(run_name)
}

/// Create a `CodeRun` CR for a remediation button click
async fn create_remediation_coderun(
    _state: &CallbackState,
    agent: Agent,
    pr_context: &PrContext,
    check_name: &str,
    check_run_id: u64,
) -> Result<String, String> {
    use kube::{api::PostParams, Api};

    let run_name = format!(
        "remediate-{}-pr{}-{}",
        agent.as_str(),
        pr_context.number,
        chrono::Utc::now().timestamp()
    );

    let coderun = serde_json::json!({
        "apiVersion": "agents.platform/v1",
        "kind": "CodeRun",
        "metadata": {
            "name": run_name,
            "namespace": "cto",
            "labels": {
                "trigger": "remediation-button",
                "agent": agent.as_str(),
                "pr-number": pr_context.number.to_string(),
                "check-run-id": check_run_id.to_string()
            }
        },
        "spec": {
            "runType": "remediation",
            "service": pr_context.repo_full_name.split('/').next_back().unwrap_or("unknown"),
            "repositoryUrl": pr_context.clone_url,
            "docsRepositoryUrl": pr_context.clone_url,
            "docsProjectDirectory": ".",
            "workingDirectory": ".",
            "githubApp": agent.github_app_name(),
            "model": "claude-opus-4-5-20251101",
            "env": {
                "PR_NUMBER": pr_context.number.to_string(),
                "PR_BRANCH": pr_context.head_branch,
                "PR_SHA": pr_context.head_sha,
                "PR_URL": pr_context.html_url,
                "REPO_FULL_NAME": pr_context.repo_full_name,
                "TRIGGER": "remediation-button",
                "FAILED_CHECK": check_name,
                "CHECK_RUN_ID": check_run_id.to_string()
            }
        }
    });

    let client = kube::Client::try_default()
        .await
        .map_err(|e| format!("Failed to create k8s client: {e}"))?;

    let api: Api<kube::api::DynamicObject> = Api::namespaced_with(
        client,
        "cto",
        &kube::api::ApiResource {
            group: "agents.platform".to_string(),
            version: "v1".to_string(),
            kind: "CodeRun".to_string(),
            api_version: "agents.platform/v1".to_string(),
            plural: "coderuns".to_string(),
        },
    );

    let obj: kube::api::DynamicObject =
        serde_json::from_value(coderun).map_err(|e| format!("Failed to serialize CodeRun: {e}"))?;

    api.create(&PostParams::default(), &obj)
        .await
        .map_err(|e| format!("Failed to create CodeRun: {e}"))?;

    Ok(run_name)
}

// =============================================================================
// Language Detection (for auto-selecting agent on check_run failures)
// =============================================================================

/// Detect the primary language from a list of changed files
#[must_use]
pub fn detect_primary_language(files: &[String]) -> Language {
    use std::collections::HashMap;

    let mut counts: HashMap<Language, usize> = HashMap::new();

    for file in files {
        let lang = Language::from_path(file);
        if lang != Language::Unknown {
            *counts.entry(lang).or_default() += 1;
        }
    }

    // Priority order for ties: Rust > Go > TypeScript > React > Python > C++ > C#
    let priority = [
        Language::Rust,
        Language::Go,
        Language::TypeScript,
        Language::React,
        Language::ReactNative,
        Language::Python,
        Language::Cpp,
        Language::CSharp,
        Language::JavaScript,
    ];

    let max_count = counts.values().max().copied().unwrap_or(0);
    if max_count == 0 {
        return Language::Unknown;
    }

    for lang in priority {
        if counts.get(&lang).copied().unwrap_or(0) == max_count {
            return lang;
        }
    }

    Language::Unknown
}

/// Select the best agent for a set of changed files
#[must_use]
pub fn select_agent_for_files(files: &[String]) -> Agent {
    let lang = detect_primary_language(files);
    lang.recommended_agent()
}

// =============================================================================
// CI Failure Button Creation
// =============================================================================

/// Handle CI failure webhook - creates a check run with remediation buttons
#[allow(clippy::too_many_lines)]
pub async fn handle_ci_failure_webhook(
    State(state): State<Arc<CallbackState>>,
    _headers: HeaderMap,
    body: Bytes,
) -> Result<Json<Value>, StatusCode> {
    info!("Received CI failure webhook");

    // Parse the payload
    let payload: Value = serde_json::from_slice(&body).map_err(|e| {
        error!("Failed to parse CI failure payload: {}", e);
        StatusCode::BAD_REQUEST
    })?;

    // Extract nested payload (from Argo Events sensor)
    let inner_payload = payload.get("payload").unwrap_or(&payload);

    if let Some(response) = maybe_dispatch_legacy_agent_webhook_to_morgan(
        &state,
        "/webhooks/github/ci-failure",
        "ci-failure",
        inner_payload,
    )
    .await?
    {
        return Ok(response);
    }

    handle_ci_failure_webhook_impl(&state, inner_payload).await
}

/// Inner implementation for CI failure webhook (callable from unified handler).
///
/// Accepts raw GitHub event bytes (no Argo Events wrapping).
pub async fn handle_ci_failure_webhook_inner(
    state: &CallbackState,
    body: &[u8],
) -> Result<Json<Value>, StatusCode> {
    let payload: Value = serde_json::from_slice(body).map_err(|e| {
        error!("Failed to parse CI failure payload: {}", e);
        StatusCode::BAD_REQUEST
    })?;
    handle_ci_failure_webhook_impl(state, &payload).await
}

/// Shared implementation for CI failure webhook processing.
/// Fetch the list of changed file paths from a GitHub PR.
async fn fetch_pr_files(
    client: &reqwest::Client,
    token: &str,
    repo: &str,
    pr_number: u64,
) -> Result<Vec<String>, StatusCode> {
    let url = format!("https://api.github.com/repos/{repo}/pulls/{pr_number}/files?per_page=100");

    let resp = client
        .get(&url)
        .header("Authorization", format!("Bearer {token}"))
        .header("Accept", "application/vnd.github+json")
        .header("User-Agent", "cto-pm-server")
        .header("X-GitHub-Api-Version", "2022-11-28")
        .send()
        .await
        .map_err(|e| {
            error!("Failed to fetch PR files: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    if !resp.status().is_success() {
        let status = resp.status();
        let body = resp.text().await.unwrap_or_default();
        error!("GitHub API error fetching PR files: {} - {}", status, body);
        return Err(StatusCode::INTERNAL_SERVER_ERROR);
    }

    let files_json: Vec<Value> = resp.json().await.map_err(|e| {
        error!("Failed to parse PR files response: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    Ok(files_json
        .iter()
        .filter_map(|f| f.get("filename").and_then(|v| v.as_str()).map(String::from))
        .collect())
}

/// Return a user-facing label for a remediation agent.
const fn remediation_label(agent: Agent) -> &'static str {
    match agent {
        Agent::Rex => "🛠️ Fix with Rex",
        Agent::Grizz => "🐻 Fix with Grizz",
        Agent::Blaze => "⚡ Fix with Blaze",
        Agent::Nova => "✨ Fix with Nova",
        Agent::Tap => "📱 Fix with Tap",
        Agent::Spark => "💻 Fix with Spark",
        Agent::Angie => "🧠 Fix with Angie",
        Agent::Vex => "🎮 Fix with Vex",
        Agent::Forge => "🔥 Fix with Forge",
        _ => "🤖 Fix with Agent",
    }
}

/// Return a description string for a remediation agent.
const fn remediation_description(agent: Agent) -> &'static str {
    match agent {
        Agent::Rex => "Rex will analyze and fix Rust issues",
        Agent::Grizz => "Grizz will analyze and fix Go issues",
        Agent::Blaze => "Blaze will analyze and fix TypeScript/React issues",
        Agent::Nova => "Nova will analyze and fix Node.js issues",
        Agent::Tap => "Tap will analyze and fix React Native issues",
        Agent::Spark => "Spark will analyze and fix Electron issues",
        Agent::Angie => "Angie will analyze and fix agent platform/orchestration issues",
        Agent::Vex => "Vex will analyze and fix Unity/C# issues",
        Agent::Forge => "Forge will analyze and fix Unreal/C++ issues",
        _ => "Agent will analyze and fix issues",
    }
}

/// Post a GitHub Check Run with the remediation button action.
async fn post_remediation_check_run(
    client: &reqwest::Client,
    token: &str,
    repo: &str,
    payload: &Value,
) -> Result<(), StatusCode> {
    let url = format!("https://api.github.com/repos/{repo}/check-runs");

    let resp = client
        .post(&url)
        .header("Authorization", format!("Bearer {token}"))
        .header("Accept", "application/vnd.github+json")
        .header("User-Agent", "cto-pm-server")
        .header("X-GitHub-Api-Version", "2022-11-28")
        .json(payload)
        .send()
        .await
        .map_err(|e| {
            error!("Failed to create check run: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    if !resp.status().is_success() {
        let status = resp.status();
        let body = resp.text().await.unwrap_or_default();
        error!("GitHub API error creating check run: {} - {}", status, body);
        return Err(StatusCode::INTERNAL_SERVER_ERROR);
    }
    Ok(())
}

async fn handle_ci_failure_webhook_impl(
    state: &CallbackState,
    inner_payload: &Value,
) -> Result<Json<Value>, StatusCode> {
    let github_token = state.github_token.as_ref().ok_or_else(|| {
        error!("No GitHub token configured for CI failure handler");
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    let event: CheckRunPayload = serde_json::from_value(inner_payload.clone()).map_err(|e| {
        error!("Failed to parse check_run payload: {}", e);
        StatusCode::BAD_REQUEST
    })?;

    let pr = event.check_run.pull_requests.first().ok_or_else(|| {
        warn!("No PR associated with check_run");
        StatusCode::BAD_REQUEST
    })?;

    let pr_number = pr.number;
    let repo_full_name = &event.repository.full_name;
    let check_run_id = event.check_run.id;
    let check_name = &event.check_run.name;

    info!(
        pr = %pr_number, repo = %repo_full_name,
        check_name = %check_name, check_run_id = %check_run_id,
        "Processing CI failure for remediation buttons"
    );

    let files = fetch_pr_files(&state.http_client, github_token, repo_full_name, pr_number).await?;
    if files.is_empty() {
        warn!("No files found for PR #{}", pr_number);
        return Ok(Json(
            json!({ "status": "skipped", "reason": "no files in PR" }),
        ));
    }

    let agent = select_agent_for_files(&files);
    let agent_name = agent.as_str();
    let label = remediation_label(agent);
    let description = remediation_description(agent);
    info!(agent = %agent_name, files_count = %files.len(), "Detected agent for remediation");

    let identifier = format!("fix-{agent_name}-pr{pr_number}-{check_run_id}");
    let check_run_name = format!("🔧 Remediation Available - {check_name}");
    let head_sha = &event.check_run.head_sha;

    let check_payload = json!({
        "name": check_run_name,
        "head_sha": head_sha,
        "status": "completed",
        "conclusion": "action_required",
        "output": {
            "title": "AI Agent Ready to Fix",
            "summary": format!("The **{check_name}** check failed. Click the button below to launch **{label}** to automatically fix the issue."),
            "text": format!(
                "## Detected Language\n\nBased on the PR files, the primary language is **{}**.\n\n## Available Agent\n\n**{label}** is ready to analyze the failure and push a fix.\n\n## How It Works\n\n1. Click the \"{label}\" button\n2. {label} will clone the repo and analyze the failure\n3. The agent will push a fix commit to this PR\n4. CI will re-run automatically",
                agent.primary_language().unwrap_or("unknown"),
            )
        },
        "actions": [{ "label": label, "description": description, "identifier": identifier }]
    });

    post_remediation_check_run(
        &state.http_client,
        github_token,
        repo_full_name,
        &check_payload,
    )
    .await?;

    info!(
        pr = %pr_number, check_name = %check_run_name, agent = %agent_name,
        "Created remediation check run with button"
    );

    Ok(Json(json!({
        "status": "ok",
        "check_run_created": check_run_name,
        "agent": agent_name,
        "pr": pr_number,
        "repo": repo_full_name
    })))
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_mentions() {
        let comment = "@5DLabs-Stitch please review the error handling in this PR";
        let mentions = parse_mentions(comment);
        assert_eq!(mentions.len(), 1);
        assert_eq!(mentions[0].agent, Agent::Stitch);
        assert!(mentions[0].instructions.contains("please review"));

        let comment2 = "@5dlabs-rex fix the clippy warnings";
        let mentions2 = parse_mentions(comment2);
        assert_eq!(mentions2.len(), 1);
        assert_eq!(mentions2[0].agent, Agent::Rex);
    }

    #[test]
    fn test_parse_button_identifier() {
        let id = "fix-rex-pr123-456789";
        let parsed = parse_button_identifier(id);
        assert!(parsed.is_some());
        let (agent, pr, check) = parsed.unwrap();
        assert_eq!(agent, Agent::Rex);
        assert_eq!(pr, 123);
        assert_eq!(check, 456_789);

        let invalid = "invalid-format";
        assert!(parse_button_identifier(invalid).is_none());
    }

    #[test]
    fn test_language_detection() {
        assert_eq!(Language::from_path("src/main.rs"), Language::Rust);
        assert_eq!(Language::from_path("cmd/server/main.go"), Language::Go);
        assert_eq!(
            Language::from_path("components/Button.tsx"),
            Language::React
        );
        assert_eq!(
            Language::from_path("src/utils/helpers.ts"),
            Language::TypeScript
        );
    }

    #[test]
    fn test_select_agent_for_files() {
        let rust_files = vec![
            "src/main.rs".to_string(),
            "src/lib.rs".to_string(),
            "Cargo.toml".to_string(),
        ];
        assert_eq!(select_agent_for_files(&rust_files), Agent::Rex);

        let mixed_files = vec![
            "src/main.rs".to_string(),
            "web/App.tsx".to_string(),
            "web/index.tsx".to_string(),
        ];
        // More React files, should pick Blaze
        assert_eq!(select_agent_for_files(&mixed_files), Agent::Blaze);
    }
}
