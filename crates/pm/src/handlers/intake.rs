//! Intake workflow handler for Linear integration.
//!
//! Handles the intake workflow triggered by delegating a PRD issue to the CTO agent.
//!
//! ## New Architecture (v2)
//!
//! The intake workflow now uses direct `CodeRun` creation instead of Argo workflows:
//!
//! 1. PM server receives Linear webhook (Morgan assigned to PRD issue)
//! 2. PM server reads project-specific `cto-config.json` from `ConfigMap`
//! 3. PM server creates `CodeRun` CR directly (no Argo workflow)
//! 4. Controller processes `CodeRun`, Morgan handles GitHub setup + intake
//! 5. Linear sidecar logs all progress to the issue
//!
//! This replaces the previous multi-step Argo workflow approach.

use anyhow::{anyhow, Context, Result};
use k8s_openapi::api::core::v1::ConfigMap;
use kube::{
    api::{Api, ObjectMeta, PostParams},
    Client as KubeClient,
};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, HashMap};
use tracing::{debug, error, info, warn};

use crate::config::{CtoConfig, IntakeConfig};
use crate::handlers::document::configmap_name_for_project;
use crate::models::{
    Issue, IssueCreateInput, IssueRelationCreateInput, IssueRelationType, Label, Project,
    ProjectCreateInput,
};
use crate::LinearClient;

/// Agent assignments for different task types.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct AgentConfig {
    /// Backend agent.
    #[serde(default)]
    pub backend: Option<String>,
    /// Frontend agent.
    #[serde(default)]
    pub frontend: Option<String>,
    /// Testing agent.
    #[serde(default)]
    pub testing: Option<String>,
    /// Quality agent.
    #[serde(default)]
    pub quality: Option<String>,
}

/// Tech stack configuration extracted from issue labels.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct TechStack {
    /// Backend framework/language.
    #[serde(default)]
    pub backend: Option<String>,
    /// Frontend framework.
    #[serde(default)]
    pub frontend: Option<String>,
    /// Programming languages.
    #[serde(default)]
    pub languages: Vec<String>,
    /// Agent assignments.
    #[serde(default)]
    pub agents: AgentConfig,
}

/// Intake request extracted from a Linear webhook.
#[derive(Debug, Clone)]
pub struct IntakeRequest {
    /// Linear session ID for activity updates.
    pub session_id: String,
    /// OAuth access token from Linear webhook (for agent API calls).
    pub access_token: Option<String>,
    /// PRD issue ID.
    pub prd_issue_id: String,
    /// PRD issue identifier (e.g., "TSK-1").
    pub prd_identifier: String,
    /// Team ID for creating task issues.
    pub team_id: String,
    /// PRD title.
    pub title: String,
    /// Project name (derived from title, used for repo creation).
    pub project_name: Option<String>,
    /// PRD content (issue description).
    pub prd_content: String,
    /// Architecture content (from linked documents).
    pub architecture_content: Option<String>,
    /// GitHub repository URL.
    pub repository_url: Option<String>,
    /// GitHub repository visibility (public or private).
    pub github_visibility: String,
    /// Source branch.
    pub source_branch: Option<String>,
    /// Tech stack from labels.
    pub tech_stack: TechStack,
    /// CTO configuration from labels/frontmatter.
    pub cto_config: CtoConfig,
    /// Existing project (if PRD issue already belongs to one).
    pub existing_project: Option<crate::models::Project>,
}

// =========================================================================
// CTO Configuration Extraction
// =========================================================================

/// Combined CLI+Model configurations from labels.
/// NOTE: Label initialization is handled by the ensure-linear-labels workflow step.
/// Format: "cli:model" in Linear - user picks ONE tag for the full config.
///
/// Tuple: (`label_name`, cli, `full_model_name`)
const KNOWN_AGENT_CONFIGS: &[(&str, &str, &str)] = &[
    // Claude CLI combinations (Anthropic models)
    ("claude:opus", "claude", "claude-opus-4-5-20251101"),
    ("claude:sonnet", "claude", "claude-sonnet-4-5-20250929"),
    ("claude:haiku", "claude", "claude-haiku-4-5-20251001"),
    // Codex CLI combinations (OpenAI models)
    ("codex:gpt5", "codex", "gpt-5.1-codex"),
    // Cursor CLI combinations (supports Claude + OpenAI)
    ("cursor:opus", "cursor", "claude-opus-4-5-20251101"),
    ("cursor:sonnet", "cursor", "claude-sonnet-4-5-20250929"),
    // OpenCode CLI combinations (Grok)
    ("opencode:grok4", "opencode", "grok-4"),
    // Gemini CLI combinations
    ("gemini:pro3", "gemini", "gemini-3-pro-preview"),
];

/// Legacy: Known CLI options (kept for backwards compatibility with flat labels)
const KNOWN_CLIS: &[&str] = &["claude", "codex", "cursor", "opencode", "gemini", "factory"];

/// Legacy: Known model shortcuts (kept for backwards compatibility with flat labels)
const KNOWN_MODEL_LABELS: &[(&str, &str)] = &[
    ("opus", "claude-opus-4-5-20251101"),
    ("sonnet", "claude-sonnet-4-5-20250929"),
    ("haiku", "claude-haiku-4-5-20251001"),
    ("gpt5", "gpt-5.1-codex"),
    ("o3", "o3"),
    ("grok4", "grok-4"),
    ("gemini3", "gemini-3-pro-preview"),
];

/// Wrapper for frontmatter parsing that contains the cto config.
#[derive(Debug, Deserialize)]
struct FrontmatterWrapper {
    cto: Option<CtoConfig>,
}

/// Parse YAML frontmatter from issue description.
///
/// Supports format:
/// ```text
/// ---
/// cto:
///   cli: cursor
///   model: claude-opus-4-20250514
/// ---
/// Actual PRD content...
/// ```
///
/// Returns `None` if no valid frontmatter is found or parsing fails.
#[must_use]
pub fn parse_cto_frontmatter(description: &str) -> Option<CtoConfig> {
    // Check for frontmatter delimiter
    let trimmed = description.trim_start();
    if !trimmed.starts_with("---") {
        return None;
    }

    // Find the closing delimiter
    let after_first = &trimmed[3..];
    let end_pos = after_first.find("\n---")?;

    // Extract the YAML content between delimiters
    let yaml_content = &after_first[..end_pos].trim();

    // Parse the YAML
    match serde_yaml::from_str::<FrontmatterWrapper>(yaml_content) {
        Ok(wrapper) => {
            if let Some(config) = wrapper.cto {
                if !config.is_empty() {
                    info!(
                        cli = ?config.cli,
                        model = ?config.model,
                        "Extracted CTO config from frontmatter"
                    );
                    return Some(config);
                }
            }
            None
        }
        Err(e) => {
            warn!(error = %e, "Failed to parse frontmatter YAML");
            None
        }
    }
}

/// Extract CTO config from issue labels.
///
/// Looks for labels in the format:
/// - `CTO CLI/claude`, `CTO CLI/cursor`, etc. (grouped labels)
/// - `claude`, `cursor`, etc. (flat labels - for CLI)
/// - `CTO Model/sonnet`, `CTO Model/opus`, etc. (grouped labels)
/// - `sonnet`, `opus`, etc. (flat labels - for model shortcuts)
#[must_use]
pub fn extract_config_from_labels(labels: &[Label]) -> CtoConfig {
    let mut config = CtoConfig::default();

    for label in labels {
        let name = label.name.to_lowercase();

        // Check for combined agent label format: "cli:model" (e.g., "claude:opus")
        // This is the preferred format - user picks ONE tag for full config
        if name.contains(':') {
            if let Some((_, cli, model)) = KNOWN_AGENT_CONFIGS
                .iter()
                .find(|(label_name, _, _)| *label_name == name)
            {
                config.cli = Some((*cli).to_string());
                config.model = Some((*model).to_string());
                // Found combined config, we're done
                break;
            }
        }

        // Legacy: Check for separate CLI label format: "CTO CLI/xxx"
        if let Some(cli) = name.strip_prefix("cto cli/") {
            let cli = cli.trim();
            if KNOWN_CLIS.contains(&cli) {
                config.cli = Some(cli.to_string());
                continue;
            }
        }

        // Legacy: Check for separate model label format: "CTO Model/xxx"
        if let Some(model_label) = name.strip_prefix("cto model/") {
            let model_label = model_label.trim();
            if let Some((_, full_model)) = KNOWN_MODEL_LABELS
                .iter()
                .find(|(short, _)| *short == model_label)
            {
                config.model = Some((*full_model).to_string());
            } else {
                // Assume it's a full model name
                config.model = Some(model_label.to_string());
            }
            continue;
        }

        // Legacy: Check for flat CLI labels
        if config.cli.is_none() && KNOWN_CLIS.contains(&name.as_str()) {
            config.cli = Some(name.clone());
            continue;
        }

        // Legacy: Check for flat model shortcut labels
        if config.model.is_none() {
            if let Some((_, full_model)) = KNOWN_MODEL_LABELS
                .iter()
                .find(|(short, _)| *short == name.as_str())
            {
                config.model = Some((*full_model).to_string());
            }
        }

        // Check for prompt style label: "cto:prompt:xxx" (e.g., "cto:prompt:minimal")
        if let Some(style) = name.strip_prefix("cto:prompt:") {
            let style = style.trim();
            if !style.is_empty() {
                config.prompt_style = Some(style.to_string());
            }
        }
    }

    if !config.is_empty() {
        info!(
            cli = ?config.cli,
            model = ?config.model,
            prompt_style = ?config.prompt_style,
            "Extracted CTO config from labels"
        );
    }

    config
}

/// Extract CTO config from issue (labels + frontmatter).
///
/// Resolution order: Description frontmatter > Labels
/// Frontmatter values override label values.
#[must_use]
pub fn extract_cto_config(issue: &Issue) -> CtoConfig {
    // 1. Extract from labels
    let mut config = extract_config_from_labels(&issue.labels);

    // 2. Parse frontmatter (overrides labels)
    if let Some(desc) = &issue.description {
        if let Some(fm_config) = parse_cto_frontmatter(desc) {
            config.merge(&fm_config);
        }
    }

    config
}

/// Extract tech stack from issue labels.
fn extract_tech_stack(labels: &[Label]) -> TechStack {
    let label_names: Vec<String> = labels.iter().map(|l| l.name.to_lowercase()).collect();

    let mut tech_stack = TechStack::default();

    // Backend detection
    if label_names.iter().any(|l| l == "rust") {
        tech_stack.backend = Some("rust".to_string());
        tech_stack.languages.push("rust".to_string());
    }
    if label_names.iter().any(|l| l == "python") {
        tech_stack.backend = Some("python".to_string());
        tech_stack.languages.push("python".to_string());
    }
    if label_names.iter().any(|l| l == "go" || l == "golang") {
        tech_stack.backend = Some("go".to_string());
        tech_stack.languages.push("go".to_string());
    }
    if label_names.iter().any(|l| l == "node" || l == "nodejs") {
        tech_stack.backend = Some("node".to_string());
    }

    // Frontend detection
    if label_names.iter().any(|l| l == "react") {
        tech_stack.frontend = Some("react".to_string());
    }
    if label_names.iter().any(|l| l == "vue") {
        tech_stack.frontend = Some("vue".to_string());
    }
    if label_names.iter().any(|l| l == "svelte") {
        tech_stack.frontend = Some("svelte".to_string());
    }
    if label_names.iter().any(|l| l == "typescript" || l == "ts") {
        tech_stack.languages.push("typescript".to_string());
    }

    tech_stack
}

/// Strip frontmatter from description content.
///
/// Returns the description with the YAML frontmatter block removed.
#[must_use]
pub fn strip_frontmatter(description: &str) -> String {
    let trimmed = description.trim_start();
    if !trimmed.starts_with("---") {
        return description.to_string();
    }

    let after_first = &trimmed[3..];
    if let Some(end_pos) = after_first.find("\n---") {
        // Skip past the closing --- and any following newline
        let rest = &after_first[end_pos + 4..];
        rest.trim_start_matches('\n').to_string()
    } else {
        description.to_string()
    }
}

/// Task from intake workflow output.
///
/// Note: Uses custom deserializers to accept both string ("1") and numeric (1) IDs,
/// since AI agents may generate either format in tasks.json.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IntakeTask {
    /// Task ID (accepts both string and numeric).
    #[serde(deserialize_with = "deserialize_string_or_number")]
    pub id: String,
    /// Task title.
    pub title: String,
    /// Task description.
    pub description: String,
    /// Detailed implementation notes.
    #[serde(default)]
    pub details: String,
    /// Dependencies (list of task IDs, accepts both string and numeric).
    #[serde(default, deserialize_with = "deserialize_string_or_number_vec")]
    pub dependencies: Vec<String>,
    /// Priority (1=urgent, 2=high, 3=normal, 4=low).
    /// Accepts both string ("medium", "high") and integer (1-4) formats.
    #[serde(
        default = "default_priority",
        deserialize_with = "deserialize_priority_flexible"
    )]
    pub priority: i32,
    /// Test strategy.
    #[serde(default, rename = "testStrategy")]
    pub test_strategy: String,
    /// Agent hint for assignment.
    #[serde(default, rename = "agentHint")]
    pub agent_hint: String,
}

/// Deserialize a value that could be either a string or a number into a String.
fn deserialize_string_or_number<'de, D>(deserializer: D) -> Result<String, D::Error>
where
    D: serde::Deserializer<'de>,
{
    use serde::de::{self, Visitor};
    use std::fmt;

    struct StringOrNumberVisitor;

    impl Visitor<'_> for StringOrNumberVisitor {
        type Value = String;

        fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
            formatter.write_str("a string or a number")
        }

        fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
        where
            E: de::Error,
        {
            Ok(value.to_string())
        }

        fn visit_string<E>(self, value: String) -> Result<Self::Value, E>
        where
            E: de::Error,
        {
            Ok(value)
        }

        fn visit_i64<E>(self, value: i64) -> Result<Self::Value, E>
        where
            E: de::Error,
        {
            Ok(value.to_string())
        }

        fn visit_u64<E>(self, value: u64) -> Result<Self::Value, E>
        where
            E: de::Error,
        {
            Ok(value.to_string())
        }
    }

    deserializer.deserialize_any(StringOrNumberVisitor)
}

/// Deserialize a Vec where each element could be either a string or a number.
fn deserialize_string_or_number_vec<'de, D>(deserializer: D) -> Result<Vec<String>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    use serde::de::Error;
    use serde::Deserialize;

    // Deserialize as a Vec of JSON values first, then convert each
    let values: Vec<serde_json::Value> = Vec::deserialize(deserializer)?;
    values
        .into_iter()
        .map(|v| match v {
            serde_json::Value::String(s) => Ok(s),
            serde_json::Value::Number(n) => Ok(n.to_string()),
            _ => Err(D::Error::custom("expected string or number")),
        })
        .collect()
}

/// Deserialize priority from either string ("medium") or integer (3).
/// Maps string priorities to Linear's integer format:
/// - "critical"/"urgent" → 1 (Urgent)
/// - "high" → 2 (High)
/// - "medium"/"normal" → 3 (Normal)
/// - "low" → 4 (Low)
fn deserialize_priority_flexible<'de, D>(deserializer: D) -> Result<i32, D::Error>
where
    D: serde::Deserializer<'de>,
{
    use serde::de::{self, Visitor};
    use std::fmt;

    struct PriorityVisitor;

    impl Visitor<'_> for PriorityVisitor {
        type Value = i32;

        fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
            formatter.write_str("a priority string or integer")
        }

        fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
        where
            E: de::Error,
        {
            Ok(match value.to_lowercase().as_str() {
                "critical" | "urgent" => 1,
                "high" => 2,
                "low" => 4,
                // "medium", "normal", "med", or any unknown string defaults to normal priority
                _ => 3,
            })
        }

        fn visit_i64<E>(self, value: i64) -> Result<Self::Value, E>
        where
            E: de::Error,
        {
            Ok(value.clamp(0, 4) as i32)
        }

        fn visit_u64<E>(self, value: u64) -> Result<Self::Value, E>
        where
            E: de::Error,
        {
            Ok(value.min(4) as i32)
        }
    }

    deserializer.deserialize_any(PriorityVisitor)
}

/// Default priority value (Normal = 3).
fn default_priority() -> i32 {
    3
}

/// Support agents that should only be used for review/audit tasks, not implementation.
const SUPPORT_AGENTS: &[&str] = &["cipher", "cleo", "tess", "atlas"];

/// Keywords that indicate a task is a review/audit task (valid for support agents).
const REVIEW_KEYWORDS: &[&str] = &[
    "review",
    "audit",
    "scan",
    "test suite",
    "testing",
    "merge",
    "integrate prs",
    "security audit",
    "security review",
    "vulnerability scan",
    "penetration test",
    "code review",
    "quality review",
];

/// Result of validating agent assignments.
#[derive(Debug, Clone, Default)]
pub struct AgentValidationResult {
    /// Warnings about potentially incorrect agent assignments.
    pub warnings: Vec<String>,
    /// Count of tasks assigned to support agents for implementation.
    pub support_agent_implementation_count: usize,
}

/// Validate agent assignments in tasks.
///
/// Returns warnings if support agents (cipher, cleo, tess, atlas) are assigned
/// to implementation tasks rather than review/audit tasks.
///
/// Support agents should ONLY be used for:
/// - `cipher`: Security audits, vulnerability scans
/// - `cleo`: Code quality reviews
/// - `tess`: Writing test suites
/// - `atlas`: Merging PRs, integration
#[must_use]
pub fn validate_agent_assignments(tasks: &[IntakeTask]) -> AgentValidationResult {
    let mut result = AgentValidationResult::default();

    for task in tasks {
        let agent = task.agent_hint.to_lowercase();

        // Check if task is assigned to a support agent
        if !SUPPORT_AGENTS.contains(&agent.as_str()) {
            continue;
        }

        // Check if the task title/description suggests it's actually a review task
        let title_lower = task.title.to_lowercase();
        let desc_lower = task.description.to_lowercase();
        let combined = format!("{title_lower} {desc_lower}");

        let is_review_task = REVIEW_KEYWORDS
            .iter()
            .any(|keyword| combined.contains(keyword));

        if !is_review_task {
            result.support_agent_implementation_count += 1;
            result.warnings.push(format!(
                "Task {}: '{}' is assigned to support agent '{}' but appears to be an implementation task, not a review/audit task. \
                 Consider using an implementation agent (bolt/rex/grizz/nova/blaze/tap/spark) instead.",
                task.id, task.title, agent
            ));
        }
    }

    // Add summary warning if many tasks are misassigned
    if result.support_agent_implementation_count > 2 {
        result.warnings.insert(
            0,
            format!(
                "⚠️ {} tasks are assigned to support agents for implementation. \
                 This is likely incorrect - support agents (cipher/cleo/tess/atlas) should only be used for review/audit tasks.",
                result.support_agent_implementation_count
            ),
        );
    }

    result
}

/// `tasks.json` structure from intake output.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TasksJson {
    /// List of generated tasks.
    pub tasks: Vec<IntakeTask>,
}

/// Result of the intake workflow.
#[derive(Debug, Clone)]
pub struct IntakeResult {
    /// Workflow name.
    pub workflow_name: String,
    /// `ConfigMap` name containing intake data.
    pub configmap_name: String,
}

/// Extract GitHub visibility setting from labels.
///
/// Returns "public" if `github:public` label is present, "private" otherwise (default).
#[must_use]
pub fn extract_github_visibility(labels: &[Label]) -> String {
    for label in labels {
        let name = label.name.to_lowercase();
        if name == "github:public" {
            return "public".to_string();
        }
    }
    // Default to private for security
    "private".to_string()
}

/// Extract intake request from a Linear issue.
///
/// This function reads PRD and architecture content from the project `ConfigMap`
/// (source of truth), falling back to parsing the issue description if the
/// `ConfigMap` doesn't exist or doesn't have the content.
///
/// This makes the workflow Linear-independent - we don't need to re-fetch
/// content from Linear documents.
pub async fn extract_intake_request(
    kube_client: &KubeClient,
    session_id: &str,
    access_token: Option<&str>,
    issue: &Issue,
) -> Result<IntakeRequest> {
    let team_id = issue
        .team
        .as_ref()
        .ok_or_else(|| anyhow!("Issue has no team"))?
        .id
        .clone();

    let raw_description = issue
        .description
        .clone()
        .ok_or_else(|| anyhow!("PRD issue has no description"))?;

    // Extract CTO config from labels and frontmatter
    let cto_config = extract_cto_config(issue);

    // Extract tech stack from labels
    let tech_stack = extract_tech_stack(&issue.labels);

    // Extract GitHub visibility from labels (default: private)
    let github_visibility = extract_github_visibility(&issue.labels);

    // Derive project name from title (sanitized for repo name)
    let project_name = sanitize_for_repo_name(&issue.title);

    // Try to read PRD, architecture, and repository from project ConfigMap (source of truth)
    let (prd_content, architecture_content, configmap_repo_url) =
        if let Some(ref project) = issue.project {
            match crate::handlers::document::read_intake_content(kube_client, &project.id).await {
                Ok(content) => {
                    info!(
                        project_id = %project.id,
                        prd_len = content.prd.len(),
                        has_arch = content.architecture.is_some(),
                        has_repo = content.repository_url.is_some(),
                        "Read intake content from project ConfigMap"
                    );
                    (content.prd, content.architecture, content.repository_url)
                }
                Err(e) => {
                    warn!(
                        project_id = %project.id,
                        error = %e,
                        "Failed to read from ConfigMap, falling back to issue description"
                    );
                    // Fallback: strip frontmatter from description
                    (strip_frontmatter(&raw_description), None, None)
                }
            }
        } else {
            // No project attached, fall back to description
            debug!("Issue has no project, extracting PRD from description");
            (strip_frontmatter(&raw_description), None, None)
        };

    // Repository URL priority: ConfigMap > description
    // If not found anywhere, the workflow will create a new repo based on the project name
    let repository_url = configmap_repo_url.or_else(|| extract_repository_url(&raw_description));
    if let Some(ref url) = repository_url {
        info!(url = %url, "Using repository URL");
    } else {
        info!(project_name = %project_name, "No repository URL found - will create new repo");
    }

    // Check if issue already belongs to a project
    if let Some(ref project) = issue.project {
        info!(
            project_id = %project.id,
            project_name = %project.name,
            "PRD issue already belongs to a project - will use existing"
        );
    }

    Ok(IntakeRequest {
        session_id: session_id.to_string(),
        access_token: access_token.map(String::from),
        prd_issue_id: issue.id.clone(),
        prd_identifier: issue.identifier.clone(),
        team_id,
        title: issue.title.clone(),
        project_name: Some(project_name),
        prd_content,
        architecture_content,
        repository_url,
        github_visibility,
        source_branch: None, // Default to main
        tech_stack,
        cto_config,
        existing_project: issue.project.clone(),
    })
}

/// Extract GitHub repository URL from issue description.
///
/// Looks for patterns like:
/// - `**Repository:** https://github.com/org/repo`
/// - `Repository: https://github.com/org/repo`
/// - `https://github.com/org/repo` (standalone URL on its own line)
#[must_use]
fn extract_repository_url(description: &str) -> Option<String> {
    use regex::Regex;

    // Pattern to match GitHub URLs (with optional .git suffix)
    let github_url_pattern = Regex::new(r"https://github\.com/[\w.-]+/[\w.-]+(?:\.git)?").ok()?;

    // First, look for explicit "Repository:" label (highest priority)
    for line in description.lines() {
        let line_lower = line.to_lowercase();
        if line_lower.contains("repository:") || line_lower.contains("repo:") {
            if let Some(captures) = github_url_pattern.find(line) {
                let url = captures.as_str().trim_end_matches(".git");
                return Some(url.to_string());
            }
        }
    }

    // Fallback: find GitHub URL that appears on its own line (not inline in prose)
    for line in description.lines() {
        let trimmed = line.trim();
        // Only match if the line is primarily just a URL (possibly with markdown formatting)
        if trimmed.starts_with("http")
            || trimmed.starts_with("**http")
            || trimmed.starts_with("[http")
        {
            if let Some(captures) = github_url_pattern.find(trimmed) {
                let url = captures.as_str().trim_end_matches(".git");
                return Some(url.to_string());
            }
        }
    }

    None
}

/// Sanitize a string for use as a GitHub repository name.
/// Converts to lowercase, replaces non-alphanumeric chars with dashes,
/// collapses consecutive dashes, and trims leading/trailing dashes.
#[must_use]
fn sanitize_for_repo_name(name: &str) -> String {
    let sanitized: String = name
        .to_lowercase()
        .chars()
        .map(|c| {
            if c.is_alphanumeric() || c == '-' {
                c
            } else {
                '-'
            }
        })
        .collect();

    // Collapse consecutive dashes and trim
    sanitized
        .split('-')
        .filter(|s| !s.is_empty())
        .collect::<Vec<_>>()
        .join("-")
}

// =========================================================================
// Project Configuration from ConfigMap
// =========================================================================

/// Project-specific configuration loaded from the Kubernetes `ConfigMap`.
///
/// This represents the settings from the `cto-config.json` document synced to the
/// `cto-config-project-{project_id}` `ConfigMap`.
#[derive(Debug, Clone, Default)]
pub struct ProjectConfig {
    /// CLI to use for the agent (claude, codex, cursor, etc.)
    pub cli: Option<String>,
    /// AI model to use
    pub model: Option<String>,
    /// GitHub App for Morgan
    pub github_app: Option<String>,
    /// Repository URL
    pub repository: Option<String>,
    /// Service name for workspace isolation
    pub service: Option<String>,
    /// Source branch
    pub source_branch: Option<String>,
    /// Morgan's tools configuration from cto-config.json agents.morgan.tools
    pub morgan_tools: Option<config::AgentTools>,
    /// Multi-model collaboration config from cto-config.json defaults.intake.multiModel
    pub multi_model: Option<MultiModelSettings>,
}

/// Multi-model collaboration settings.
#[derive(Debug, Clone, Default)]
pub struct MultiModelSettings {
    /// Whether multi-model is enabled.
    pub enabled: bool,
    /// Generator provider (claude, minimax, codex).
    pub generator: String,
    /// Critic provider (claude, minimax, codex).
    pub critic: String,
    /// Maximum refinement iterations.
    pub max_refinements: u32,
    /// Critic threshold (0.0-1.0).
    pub critic_threshold: f32,
}

/// Read project-specific configuration from the Kubernetes `ConfigMap`.
///
/// Looks up the `cto-config-project-{project_id}` `ConfigMap` in the `cto` namespace
/// and extracts the relevant settings for the intake workflow.
///
/// Returns `None` if the `ConfigMap` doesn't exist or can't be parsed.
pub async fn read_project_config(
    kube_client: &KubeClient,
    project_id: &str,
) -> Option<ProjectConfig> {
    let configmap_name = configmap_name_for_project(project_id);
    let api: Api<ConfigMap> = Api::namespaced(kube_client.clone(), "cto");

    let cm = match api.get(&configmap_name).await {
        Ok(cm) => cm,
        Err(e) => {
            debug!(
                configmap_name = %configmap_name,
                error = %e,
                "Project ConfigMap not found (will use defaults)"
            );
            return None;
        }
    };

    let data = cm.data?;
    let json_content = data.get("cto-config.json")?;

    match config::CtoConfig::from_json(json_content) {
        Ok(config) => {
            // Extract Morgan's tools configuration if present
            let morgan_tools = config.agents.get("morgan").map(|agent| {
                debug!(
                    remote_tools = ?agent.tools.remote,
                    "Extracted Morgan's tools from project config"
                );
                agent.tools.clone()
            });

            // Extract multi-model configuration if enabled
            let multi_model = if config.defaults.intake.multi_model.enabled {
                let mm = &config.defaults.intake.multi_model;
                info!(
                    generator = %mm.generator,
                    critic = %mm.critic,
                    max_refinements = mm.max_refinements,
                    "Multi-model collaboration enabled in project config"
                );
                Some(MultiModelSettings {
                    enabled: true,
                    generator: mm.generator.clone(),
                    critic: mm.critic.clone(),
                    max_refinements: mm.max_refinements,
                    critic_threshold: mm.critic_threshold,
                })
            } else {
                None
            };

            info!(
                configmap_name = %configmap_name,
                cli = ?config.defaults.intake.cli,
                model = ?config.defaults.intake.models.primary,
                has_morgan_tools = morgan_tools.is_some(),
                has_multi_model = multi_model.is_some(),
                "Loaded project config from ConfigMap"
            );

            Some(ProjectConfig {
                cli: Some(config.defaults.intake.cli),
                model: Some(config.defaults.intake.models.primary),
                github_app: Some(config.defaults.intake.github_app),
                repository: if config.defaults.play.repository.is_empty() {
                    None
                } else {
                    Some(config.defaults.play.repository)
                },
                service: if config.defaults.play.service.is_empty() {
                    None
                } else {
                    Some(config.defaults.play.service)
                },
                source_branch: Some(config.defaults.intake.source_branch),
                morgan_tools,
                multi_model,
            })
        }
        Err(e) => {
            warn!(
                configmap_name = %configmap_name,
                error = %e,
                "Failed to parse project config JSON (will use defaults)"
            );
            None
        }
    }
}

// =========================================================================
// CodeRun Direct Creation (New Architecture)
// =========================================================================

/// Submit an intake `CodeRun` directly to Kubernetes.
///
/// This is the new architecture that replaces the Argo workflow approach.
/// Benefits:
/// - Single `CodeRun` with Linear sidecar logging ALL progress
/// - No Argo workflow needed (direct `CodeRun` creation)
/// - Project-specific `cto-config.json` settings used
/// - Simpler architecture
///
/// # Arguments
/// * `kube_client` - Kubernetes client
/// * `namespace` - Kubernetes namespace
/// * `request` - Intake request with PRD content and metadata
/// * `config` - Server-side intake configuration (defaults)
///
/// # Returns
/// `IntakeResult` with the `CodeRun` name and `ConfigMap` name
#[allow(clippy::too_many_lines)] // Complex function not easily split
pub async fn submit_intake_coderun(
    kube_client: &KubeClient,
    namespace: &str,
    request: &IntakeRequest,
    config: &IntakeConfig,
) -> Result<IntakeResult> {
    let timestamp = chrono::Utc::now().timestamp();
    let workflow_project_name = sanitize_project_name(&request.prd_identifier);
    let configmap_name = format!("intake-linear-{workflow_project_name}-{timestamp}");

    info!(
        prd_identifier = %request.prd_identifier,
        project_id = ?request.existing_project.as_ref().map(|p| &p.id),
        "Submitting intake CodeRun (new architecture)"
    );

    // Try to read project-specific config from ConfigMap
    let project_config = if let Some(project) = &request.existing_project {
        read_project_config(kube_client, &project.id).await
    } else {
        None
    };

    // Use sanitized project name for repo creation (strips [PRD] prefix, normalizes)
    // Falls back to title if project_name wasn't set
    let project_name_for_repo = request.project_name.as_deref().unwrap_or(&request.title);

    // Determine CLI and model - priority: project config > issue labels/frontmatter > server defaults
    let cli = project_config
        .as_ref()
        .and_then(|c| c.cli.clone())
        .or_else(|| request.cto_config.cli.clone())
        .unwrap_or_else(|| config.cli.clone());

    let primary_model = project_config
        .as_ref()
        .and_then(|c| c.model.clone())
        .or_else(|| request.cto_config.model.clone())
        .unwrap_or_else(|| config.primary_model.clone());

    let github_app = project_config
        .as_ref()
        .and_then(|c| c.github_app.clone())
        .unwrap_or_else(|| config.github_app.clone());

    let source_branch = project_config
        .as_ref()
        .and_then(|c| c.source_branch.clone())
        .or_else(|| request.source_branch.clone())
        .unwrap_or_else(|| "main".to_string());

    // Repository URL from request first (explicit URL), then project config
    // Note: We filter out placeholder values like "unnamed-project" since intake
    // should create a new repository if no real URL is provided
    let repository_url = request
        .repository_url
        .clone()
        .or_else(|| project_config.as_ref().and_then(|c| c.repository.clone()))
        .filter(|url| !url.contains("unnamed-project") && !url.is_empty())
        .unwrap_or_default();

    let service_name = project_config
        .as_ref()
        .and_then(|c| c.service.clone())
        .unwrap_or_else(|| sanitize_for_repo_name(project_name_for_repo));

    // Extract Morgan's tools if configured
    let morgan_tools = project_config
        .as_ref()
        .and_then(|c| c.morgan_tools.as_ref());

    // Extract multi-model config if enabled
    let multi_model = project_config.as_ref().and_then(|c| c.multi_model.as_ref());

    info!(
        cli = %cli,
        model = %primary_model,
        github_app = %github_app,
        repository_url = %repository_url,
        service = %service_name,
        has_morgan_tools = morgan_tools.is_some(),
        has_multi_model = multi_model.is_some(),
        "Using intake configuration"
    );

    // Project ID for linking to project ConfigMap
    let project_id = request
        .existing_project
        .as_ref()
        .map_or("", |p| p.id.as_str());

    // Create ConfigMap with PRD content
    let config_json = serde_json::json!({
        "project_name": project_name_for_repo,
        "repository_url": repository_url,
        "github_app": github_app,
        "primary_model": primary_model,
        "research_model": config.research_model,
        "fallback_model": config.fallback_model,
        "model": primary_model,
        "num_tasks": config.num_tasks,
        "expand_tasks": config.expand_tasks,
        "analyze_complexity": config.analyze_complexity,
        "enrich_context": config.enrich_context,
        "include_codebase": config.include_codebase,
        "cli": cli,
        "linear_session_id": request.session_id,
        "linear_issue_id": request.prd_issue_id,
        "linear_issue_identifier": request.prd_identifier,
        "linear_team_id": request.team_id,
        "linear_project_id": project_id,
        // Morgan's tools from project cto-config.json
        "morgan_tools": morgan_tools.map(|t| serde_json::json!({
            "remote": t.remote,
            "localServers": t.local_servers,
        })),
    });

    let mut data = BTreeMap::new();
    data.insert("prd.txt".to_string(), request.prd_content.clone());
    data.insert(
        "architecture.md".to_string(),
        request.architecture_content.clone().unwrap_or_default(),
    );
    data.insert("config.json".to_string(), config_json.to_string());

    let configmap = ConfigMap {
        metadata: ObjectMeta {
            name: Some(configmap_name.clone()),
            namespace: Some(namespace.to_string()),
            labels: Some(BTreeMap::from([
                (
                    "app.kubernetes.io/name".to_string(),
                    "cto-intake".to_string(),
                ),
                (
                    "app.kubernetes.io/component".to_string(),
                    "intake".to_string(),
                ),
                ("cto.5dlabs.io/source".to_string(), "linear".to_string()),
                (
                    "cto.5dlabs.io/linear-issue".to_string(),
                    request.prd_identifier.clone(),
                ),
            ])),
            // Add TTL annotation for automatic cleanup (4 hours)
            annotations: Some(BTreeMap::from([(
                "cto.5dlabs.io/ttl".to_string(),
                "14400".to_string(),
            )])),
            ..Default::default()
        },
        data: Some(data),
        ..Default::default()
    };

    let cm_api: Api<ConfigMap> = Api::namespaced(kube_client.clone(), namespace);
    cm_api
        .create(&PostParams::default(), &configmap)
        .await
        .context("Failed to create intake ConfigMap")?;

    info!(configmap_name = %configmap_name, "Created intake ConfigMap");

    // Generate unique CodeRun name
    let name_suffix = workflow_project_name
        .chars()
        .take(20)
        .collect::<String>()
        .trim_matches('-')
        .to_string();
    let coderun_name = format!("intake-{name_suffix}-{timestamp}");

    // Build CodeRun CR manifest
    let project_id = request
        .existing_project
        .as_ref()
        .map_or("", |p| p.id.as_str());

    // Build tools-config annotation from Morgan's tools if available
    let tools_config_annotation = morgan_tools.map(|t| {
        serde_json::json!({
            "remote": t.remote,
            "localServers": t.local_servers
        })
        .to_string()
    });

    let coderun_json = serde_json::json!({
        "apiVersion": "agents.platform/v1",
        "kind": "CodeRun",
        "metadata": {
            "name": coderun_name,
            "namespace": namespace,
            "labels": {
                "workflow-type": "intake",
                "project-name": name_suffix,
                "github-app": github_app,
                "cto.5dlabs.io/linear-issue": request.prd_identifier
            },
            "annotations": {
                // Pass Morgan's tools to controller via annotation
                "agents.platform/tools-config": tools_config_annotation.unwrap_or_default()
            }
        },
        "spec": {
            "runType": "intake",
            "service": service_name,
            "repositoryUrl": repository_url,
            "docsRepositoryUrl": repository_url,
            "workingDirectory": ".",
            "githubApp": github_app,
            "model": primary_model,
            "enableDocker": false,
            "skillsUrl": config.skills_repo,
            "env": {
                "PROJECT_NAME": project_name_for_repo,
                "REPOSITORY_URL": repository_url,
                "NUM_TASKS": config.num_tasks.to_string(),
                "EXPAND_TASKS": config.expand_tasks.to_string(),
                "ANALYZE_COMPLEXITY": config.analyze_complexity.to_string(),
                "INTAKE_CLI": cli,
                "INTAKE_CONFIGMAP": configmap_name,
                "LINEAR_PROJECT_ID": project_id,
                // GitHub visibility for repo creation
                "GITHUB_VISIBILITY": request.github_visibility,
                "GITHUB_DEFAULT_ORG": config.github_default_org.as_deref().unwrap_or("5dlabs"),
                // Source branch configuration
                "SOURCE_BRANCH": source_branch,
                // Webhook callback URL for automatic play workflow triggers
                "WEBHOOK_CALLBACK_URL": config.webhook_callback_url.as_deref().unwrap_or(""),
                // Extended thinking configuration for task generation
                "TASKS_EXTENDED_THINKING": config.extended_thinking.to_string(),
                "TASKS_THINKING_BUDGET": config.thinking_budget.map_or(String::new(), |b| b.to_string()),
                // Multi-model collaboration configuration
                "TASKS_MULTI_MODEL": multi_model.map_or("false".to_string(), |_| "true".to_string()),
                "MULTI_MODEL_GENERATOR": multi_model.map_or(String::new(), |m| m.generator.clone()),
                "MULTI_MODEL_CRITIC": multi_model.map_or(String::new(), |m| m.critic.clone()),
                "MULTI_MODEL_MAX_REFINEMENTS": multi_model.map_or(String::new(), |m| m.max_refinements.to_string()),
                "MULTI_MODEL_CRITIC_THRESHOLD": multi_model.map_or(String::new(), |m| m.critic_threshold.to_string())
            },
            "linearIntegration": {
                "enabled": true,
                "sessionId": request.session_id,
                "accessToken": request.access_token,
                "issueId": request.prd_issue_id,
                "teamId": request.team_id,
                "projectId": project_id
            }
        }
    });

    // Create CodeRun via kube client (works both in-cluster and locally with kubeconfig)
    let coderun_gvk = kube::api::GroupVersionKind::gvk("agents.platform", "v1", "CodeRun");
    let coderun_api_resource = kube::api::ApiResource::from_gvk(&coderun_gvk);
    let coderuns: Api<kube::api::DynamicObject> =
        Api::namespaced_with(kube_client.clone(), namespace, &coderun_api_resource);

    let coderun_obj: kube::api::DynamicObject =
        serde_json::from_value(coderun_json).context("Failed to parse CodeRun JSON")?;

    coderuns
        .create(&PostParams::default(), &coderun_obj)
        .await
        .context("Failed to create CodeRun")?;

    info!(coderun_name = %coderun_name, "Created intake CodeRun");

    Ok(IntakeResult {
        workflow_name: coderun_name,
        configmap_name,
    })
}

/// Submit an intake workflow to Kubernetes.
///
/// DEPRECATED: This function uses the old Argo workflow approach.
/// New code should use `submit_intake_coderun` instead.
///
/// Creates a `ConfigMap` with PRD content and submits an Argo workflow.
///
/// Creates a `ConfigMap` with PRD content and submits an Argo workflow.
#[allow(clippy::too_many_lines)] // Complex function not easily split
pub async fn submit_intake_workflow(
    kube_client: &KubeClient,
    namespace: &str,
    request: &IntakeRequest,
    config: &IntakeConfig,
) -> Result<IntakeResult> {
    let timestamp = chrono::Utc::now().timestamp();
    // Use sanitized identifier for ConfigMap/workflow naming
    let workflow_project_name = sanitize_project_name(&request.prd_identifier);
    let configmap_name = format!("intake-linear-{workflow_project_name}-{timestamp}");
    let workflow_name = format!("intake-linear-{workflow_project_name}-{timestamp}");

    // Use issue title for new repo creation (the workflow will sanitize it)
    // This is passed as project-name parameter to the workflow
    let project_name_for_repo = &request.title;

    // Pass empty string if no repository URL - workflow will create a new repo
    let repository_url = request.repository_url.clone().unwrap_or_default();
    let source_branch = request
        .source_branch
        .clone()
        .unwrap_or_else(|| "main".to_string());

    // Apply CTO config overrides from labels/frontmatter
    let cli = request.cto_config.cli.as_deref().unwrap_or(&config.cli);
    let primary_model = request
        .cto_config
        .model
        .as_deref()
        .unwrap_or(&config.primary_model);

    if !request.cto_config.is_empty() {
        info!(
            cli = %cli,
            model = %primary_model,
            "Using CTO config overrides from issue"
        );
    }

    // Prepare config JSON for the workflow.
    let config_json = serde_json::json!({
        "project_name": project_name_for_repo,
        "repository_url": repository_url,
        "github_app": config.github_app,
        "primary_model": primary_model,
        "research_model": config.research_model,
        "fallback_model": config.fallback_model,
        "primary_provider": config.primary_provider,
        "research_provider": config.research_provider,
        "fallback_provider": config.fallback_provider,
        "model": primary_model,
        "num_tasks": config.num_tasks,
        "expand_tasks": config.expand_tasks,
        "analyze_complexity": config.analyze_complexity,
        "docs_model": config.docs_model,
        "enrich_context": config.enrich_context,
        "include_codebase": config.include_codebase,
        "cli": cli,
        // Linear-specific metadata for callbacks.
        "linear_session_id": request.session_id,
        "linear_issue_id": request.prd_issue_id,
        "linear_issue_identifier": request.prd_identifier,
        "linear_team_id": request.team_id,
    });

    // Create ConfigMap with PRD content.
    let mut data = BTreeMap::new();
    data.insert("prd.txt".to_string(), request.prd_content.clone());
    data.insert(
        "architecture.md".to_string(),
        request.architecture_content.clone().unwrap_or_default(),
    );
    data.insert("config.json".to_string(), config_json.to_string());

    let configmap = ConfigMap {
        metadata: ObjectMeta {
            name: Some(configmap_name.clone()),
            namespace: Some(namespace.to_string()),
            labels: Some(BTreeMap::from([
                (
                    "app.kubernetes.io/name".to_string(),
                    "cto-intake".to_string(),
                ),
                (
                    "app.kubernetes.io/component".to_string(),
                    "intake".to_string(),
                ),
                ("cto.5dlabs.io/source".to_string(), "linear".to_string()),
                (
                    "cto.5dlabs.io/linear-issue".to_string(),
                    request.prd_identifier.clone(),
                ),
            ])),
            ..Default::default()
        },
        data: Some(data),
        ..Default::default()
    };

    let cm_api: Api<ConfigMap> = Api::namespaced(kube_client.clone(), namespace);
    cm_api
        .create(&PostParams::default(), &configmap)
        .await
        .context("Failed to create intake ConfigMap")?;

    info!(configmap_name = %configmap_name, "Created intake ConfigMap");

    // Build Workflow manifest that references the WorkflowTemplate.
    // We use direct K8s API calls instead of argo submit or kubectl because:
    // 1. argo CLI crashes with nil pointer dereference when using --from workflowtemplate
    // 2. kubectl isn't available in the PM container
    let workflow_json = serde_json::json!({
        "apiVersion": "argoproj.io/v1alpha1",
        "kind": "Workflow",
        "metadata": {
            "name": workflow_name,
            "namespace": namespace,
            "labels": {
                "linear-session": request.session_id,
                "cto.5dlabs.io/linear-issue": request.prd_identifier,
                "cto.5dlabs.io/agent-type": "intake"
            }
        },
        "spec": {
            "workflowTemplateRef": {
                "name": "project-intake"
            },
            "arguments": {
                "parameters": [
                    {"name": "configmap-name", "value": configmap_name},
                    {"name": "project-name", "value": project_name_for_repo},
                    {"name": "repository-url", "value": repository_url},
                    {"name": "source-branch", "value": source_branch},
                    {"name": "github-app", "value": config.github_app},
                    {"name": "github-default-org", "value": config.github_default_org.as_deref().unwrap_or("5dlabs")},
                    {"name": "github-visibility", "value": request.github_visibility},
                    {"name": "webhook-callback-url", "value": config.webhook_callback_url.as_deref().unwrap_or("")},
                    {"name": "primary-model", "value": primary_model},
                    {"name": "research-model", "value": config.research_model},
                    {"name": "fallback-model", "value": config.fallback_model},
                    {"name": "primary-provider", "value": config.primary_provider},
                    {"name": "research-provider", "value": config.research_provider},
                    {"name": "fallback-provider", "value": config.fallback_provider},
                    {"name": "num-tasks", "value": config.num_tasks.to_string()},
                    {"name": "expand-tasks", "value": config.expand_tasks.to_string()},
                    {"name": "analyze-complexity", "value": config.analyze_complexity.to_string()},
                    {"name": "docs-model", "value": config.docs_model},
                    {"name": "enrich-context", "value": config.enrich_context.to_string()},
                    {"name": "include-codebase", "value": config.include_codebase.to_string()},
                    {"name": "cli", "value": cli},
                    {"name": "linear-session-id", "value": request.session_id},
                    {"name": "linear-issue-id", "value": request.prd_issue_id},
                    {"name": "linear-issue-identifier", "value": request.prd_identifier},
                    {"name": "linear-team-id", "value": request.team_id},
                    {"name": "linear-project-id", "value": request.existing_project.as_ref().map_or("", |p| p.id.as_str())},
                    {"name": "runtime-image", "value": config.runtime_image}
                ]
            }
        }
    });

    // Submit via kube client (works both in-cluster and locally with kubeconfig)
    let workflow_gvk = kube::api::GroupVersionKind::gvk("argoproj.io", "v1alpha1", "Workflow");
    let workflow_api_resource = kube::api::ApiResource::from_gvk(&workflow_gvk);
    let workflows: Api<kube::api::DynamicObject> =
        Api::namespaced_with(kube_client.clone(), namespace, &workflow_api_resource);

    let workflow_obj: kube::api::DynamicObject =
        serde_json::from_value(workflow_json).context("Failed to parse Workflow JSON")?;

    workflows
        .create(&PostParams::default(), &workflow_obj)
        .await
        .context("Failed to create Argo workflow")?;

    info!(workflow_name = %workflow_name, "Submitted intake workflow");

    Ok(IntakeResult {
        workflow_name,
        configmap_name,
    })
}

/// Resolve agent hints to Linear user IDs for delegate assignment.
///
/// For each unique agent hint in the task list, search Linear's workspace users
/// for a matching agent app (e.g., "rex" → "5DLabs-Rex"). Returns a map of
/// lowercase agent name → Linear user ID.
///
/// This follows the same pattern used for Morgan auto-assignment in `handle_intake_setup`.
pub async fn resolve_agent_delegates(
    client: &LinearClient,
    tasks: &[IntakeTask],
) -> HashMap<String, String> {
    let mut delegate_map: HashMap<String, String> = HashMap::new();

    // Collect unique agent hints
    let unique_agents: std::collections::HashSet<String> = tasks
        .iter()
        .filter(|t| !t.agent_hint.is_empty())
        .map(|t| t.agent_hint.to_lowercase())
        .collect();

    for agent_name in &unique_agents {
        match client.search_users_by_name(agent_name).await {
            Ok(users) => {
                // Match by name pattern: "5DLabs-{Agent}" or "{Agent}" (case-insensitive)
                let expected_full = format!("5dlabs-{agent_name}");
                let found = users.iter().find(|u| {
                    let lower = u.name.to_lowercase();
                    lower == expected_full || lower == *agent_name || lower.contains(agent_name)
                });
                if let Some(user) = found {
                    info!(
                        agent = %agent_name,
                        linear_user_id = %user.id,
                        linear_user_name = %user.name,
                        "Resolved agent delegate"
                    );
                    delegate_map.insert(agent_name.clone(), user.id.clone());
                } else {
                    warn!(
                        agent = %agent_name,
                        candidates = ?users.iter().map(|u| &u.name).collect::<Vec<_>>(),
                        "Agent not found in Linear workspace users"
                    );
                }
            }
            Err(e) => {
                warn!(
                    agent = %agent_name,
                    error = %e,
                    "Failed to search Linear users for agent delegate"
                );
            }
        }
    }

    info!(
        resolved = delegate_map.len(),
        total = unique_agents.len(),
        "Agent delegate resolution complete"
    );

    delegate_map
}

/// Create Linear issues from intake tasks.
///
/// This function creates sub-issues under the PRD issue for each generated task.
pub async fn create_task_issues(
    client: &LinearClient,
    request: &IntakeRequest,
    tasks: &[IntakeTask],
) -> Result<HashMap<String, String>> {
    create_task_issues_with_project(client, request, tasks, None, None).await
}

/// Create Linear issues for generated tasks, optionally linked to a project.
///
/// If `agent_delegates` is provided, agents are auto-assigned as delegates on their issues.
/// If not provided, issues are created without delegates (legacy behavior).
#[allow(clippy::too_many_lines)] // Complex function not easily split
pub async fn create_task_issues_with_project(
    client: &LinearClient,
    request: &IntakeRequest,
    tasks: &[IntakeTask],
    project_id: Option<&str>,
    agent_delegates: Option<&HashMap<String, String>>,
) -> Result<HashMap<String, String>> {
    let mut task_issue_map: HashMap<String, String> = HashMap::new();

    // Validate agent assignments and log warnings
    let validation = validate_agent_assignments(tasks);
    if !validation.warnings.is_empty() {
        warn!(
            support_agent_count = validation.support_agent_implementation_count,
            "Agent assignment validation warnings detected"
        );
        for warning in &validation.warnings {
            warn!(warning = %warning, "Agent assignment issue");
        }
    }

    // Get workflow states for the team.
    // Prefer "Ready" state (created by ensure_play_workflow_states) for new tasks,
    // falling back to any "unstarted" state if "Ready" doesn't exist.
    let states = client.get_team_workflow_states(&request.team_id).await?;
    let initial_state = states
        .iter()
        .find(|s| s.name == "Ready")
        .or_else(|| states.iter().find(|s| s.state_type == "unstarted"))
        .ok_or_else(|| anyhow!("No unstarted state found for team"))?;

    // Get or create labels.
    let play_task_label = client
        .get_or_create_label(&request.team_id, "task:play")
        .await?;

    // Get or create agent:pending label for new tasks
    let agent_pending_label = client
        .get_or_create_label(&request.team_id, "agent:pending")
        .await?;

    info!(
        task_count = tasks.len(),
        prd_issue = %request.prd_identifier,
        project_id = ?project_id,
        "Creating task issues in project"
    );

    // Create issues for each task.
    // Tasks are linked to the project (not as sub-issues of the PRD).
    // This allows proper use of Linear's project board view with workflow states.
    for task in tasks {
        let priority_label_name = match task.priority {
            1 => "priority:urgent",
            2 => "priority:high",
            4 | 5 => "priority:low",
            _ => "priority:normal",
        };

        let priority_label = client
            .get_or_create_label(&request.team_id, priority_label_name)
            .await
            .ok();

        // Format task description with details.
        let description = format_task_description(task);

        // Resolve delegate_id from agent hint if agent_delegates map is available.
        let delegate_id = agent_delegates
            .and_then(|m| m.get(&task.agent_hint.to_lowercase()))
            .cloned();

        // Determine appropriate labels based on whether agent is assigned
        let mut label_ids = vec![play_task_label.id.clone()];
        if delegate_id.is_none() {
            // Only mark as pending if no agent was resolved
            label_ids.push(agent_pending_label.id.clone());
        }

        if let Some(label) = priority_label {
            label_ids.push(label.id);
        }

        let has_delegate = delegate_id.is_some();

        // Create as standalone issue linked to project (not as sub-issue of PRD).
        // This enables proper board view in Linear with workflow state columns.
        let input = IssueCreateInput {
            team_id: request.team_id.clone(),
            title: format!("Task {}: {}", task.id, task.title),
            description: Some(description),
            parent_id: None, // Not a sub-issue - linked via project instead
            priority: Some(task.priority),
            label_ids: Some(label_ids),
            project_id: project_id.map(String::from),
            state_id: Some(initial_state.id.clone()),
            delegate_id,
        };

        match client.create_issue(input).await {
            Ok(issue) => {
                info!(
                    task_id = %task.id,
                    issue_identifier = %issue.identifier,
                    agent_hint = %task.agent_hint,
                    delegated = has_delegate,
                    "Created task issue"
                );
                task_issue_map.insert(task.id.clone(), issue.id);
            }
            Err(e) => {
                error!(task_id = %task.id, error = %e, "Failed to create task issue");
            }
        }
    }

    // Create dependency relationships.
    for task in tasks {
        if task.dependencies.is_empty() {
            continue;
        }

        let Some(issue_id) = task_issue_map.get(&task.id) else {
            continue;
        };

        for dep_id in &task.dependencies {
            let Some(dep_issue_id) = task_issue_map.get(dep_id) else {
                warn!(
                    task_id = %task.id,
                    dep_id = %dep_id,
                    "Dependency task issue not found"
                );
                continue;
            };

            // Linear only supports "blocks", not "blockedBy", so invert the relationship:
            // Instead of "issue is blocked by dep", we create "dep blocks issue"
            let input = IssueRelationCreateInput {
                issue_id: dep_issue_id.clone(),
                related_issue_id: issue_id.clone(),
                relation_type: IssueRelationType::Blocks,
            };

            if let Err(e) = client.create_issue_relation(input).await {
                warn!(
                    task_id = %task.id,
                    dep_id = %dep_id,
                    error = %e,
                    "Failed to create dependency relation"
                );
            }
        }
    }

    Ok(task_issue_map)
}

/// Play workflow phase states configuration.
///
/// These states map to the play workflow phases with 12 agents:
/// - Infrastructure: Bolt (Task 1) - DB, cache, storage setup
/// - Implementation: Rex/Grizz/Nova (backend), Blaze/Tap/Spark (frontend)
/// - Quality: Cleo - Code review, linting
/// - Security: Cipher - Security audit
/// - Testing: Tess - Test coverage
/// - Integration: Atlas - PR merge preparation
/// - Deployment: Bolt (final) - Production release
pub const PLAY_WORKFLOW_STATES: &[(&str, &str, &str)] = &[
    // (name, type, color)
    ("Ready", "unstarted", "#6B7280"),        // Gray - PRD approved
    ("Infrastructure", "started", "#10B981"), // Green - Bolt Task 1
    ("Implementation", "started", "#3B82F6"), // Blue - Rex/Blaze/etc
    ("Quality", "started", "#8B5CF6"),        // Purple - Cleo
    ("Security", "started", "#6366F1"),       // Indigo - Cipher
    ("Testing", "started", "#EC4899"),        // Pink - Tess
    ("Integration", "started", "#F59E0B"),    // Amber - Atlas
    ("Deployment", "started", "#059669"),     // Emerald - Bolt final
];

/// Ensure play workflow states exist for a team.
///
/// Creates the phase-based workflow states if they don't already exist.
/// This enables a board view with columns for each play phase:
///
/// ```text
/// Backlog → Ready → Infrastructure → Implementation → Quality → Security → Testing → Integration → Deployment → Done
/// ```
pub async fn ensure_play_workflow_states(client: &LinearClient, team_id: &str) -> Result<()> {
    info!(team_id = %team_id, "Ensuring play workflow states exist");

    for (name, state_type, color) in PLAY_WORKFLOW_STATES {
        match client
            .get_or_create_workflow_state(team_id, name, state_type, color)
            .await
        {
            Ok(state) => {
                info!(
                    state_id = %state.id,
                    state_name = %state.name,
                    "Workflow state ready"
                );
            }
            Err(e) => {
                // Log but don't fail - state might already exist with different casing
                warn!(
                    state_name = %name,
                    error = %e,
                    "Failed to create workflow state (may already exist)"
                );
            }
        }
    }

    Ok(())
}

/// Create a Linear project for an intake request.
///
/// Creates a project linked to the PRD issue's team, with appropriate
/// description and metadata. Also ensures the play workflow states exist
/// for the board view.
pub async fn create_intake_project(
    client: &LinearClient,
    request: &IntakeRequest,
    task_count: usize,
) -> Result<Project> {
    // Ensure play workflow states exist for the team
    if let Err(e) = ensure_play_workflow_states(client, &request.team_id).await {
        warn!(
            error = %e,
            "Failed to ensure play workflow states (continuing with project creation)"
        );
    }

    // Determine project name from PRD title
    let project_name = derive_project_name(&request.title);

    let description = format!(
        "## Project Overview\n\n\
         Generated from PRD: **{}** ({})\n\n\
         This project contains {} tasks for implementation.\n\n\
         ## Board View\n\n\
         Switch to **Board view** to see tasks organized by play phase:\n\n\
         ```\n\
         Backlog → Ready → 🔧 Implementation → 🔍 Quality → 🔗 Integration → 🚀 Deployment → Done\n\
         ```\n\n\
         ### Play Phases\n\
         - **🔧 Implementation**: Rex/Blaze building backend/frontend\n\
         - **🔍 Quality**: Cleo reviews, Tess tests, Cipher secures\n\
         - **🔗 Integration**: Stitch reviews, Atlas merges\n\
         - **🚀 Deployment**: Bolt deploys to production\n\n\
         ---\n\n\
         *Created by CTO Agent intake workflow*",
        request.title, request.prd_identifier, task_count
    );

    // Try to find the "Play Workflow" template for consistent project structure
    let template_id = match client.find_project_template_by_name("Play Workflow").await {
        Ok(Some(template)) => {
            info!(template_id = %template.id, "Using Play Workflow template");
            Some(template.id)
        }
        Ok(None) => {
            debug!("Play Workflow template not found, creating project without template");
            None
        }
        Err(e) => {
            warn!(error = %e, "Failed to look up project template, continuing without");
            None
        }
    };

    // Try to find "Planned" project status for initial project state
    let status_id = match client.find_project_status_by_type("planned").await {
        Ok(Some(status)) => {
            info!(status_id = %status.id, status_name = %status.name, "Using 'Planned' project status");
            Some(status.id)
        }
        Ok(None) => {
            debug!("No 'planned' type project status found, project will use default status");
            None
        }
        Err(e) => {
            warn!(error = %e, "Failed to look up project status, continuing without");
            None
        }
    };

    let input = ProjectCreateInput {
        name: project_name,
        description: Some(description),
        team_ids: Some(vec![request.team_id.clone()]),
        lead_id: None,
        target_date: None,
        template_id,
        status_id,
    };

    let project = client.create_project(input).await?;

    info!(
        project_id = %project.id,
        project_name = %project.name,
        prd = %request.prd_identifier,
        "Created Linear project for intake"
    );

    Ok(project)
}

/// Create a CTO config document for a project.
///
/// Creates a `cto-config.json` document associated with the project,
/// containing project-specific configuration for Play workflows.
pub async fn create_project_cto_config_document(
    client: &LinearClient,
    project: &Project,
    request: &IntakeRequest,
) -> Result<crate::models::Document> {
    let config_content = generate_project_cto_config(request);

    // Wrap JSON in markdown code fence for better display in Linear
    let document_content = format!(
        "# CTO Configuration\n\n\
         Project-specific configuration for Play workflows.\n\n\
         **Repository:** {}\n\
         **Service:** {}\n\n\
         ```json\n{}\n```",
        request.repository_url.as_deref().unwrap_or("(not set)"),
        derive_service_name(&project.name),
        config_content
    );

    let input = crate::models::DocumentCreateInput {
        title: "cto-config.json".to_string(),
        content: Some(document_content),
        project_id: Some(project.id.clone()),
        issue_id: None,
        icon: Some("⚙️".to_string()),
        color: None,
    };

    let document = client.create_document(input).await?;

    info!(
        document_id = %document.id,
        document_url = ?document.url,
        project_id = %project.id,
        "Created CTO config document for project"
    );

    Ok(document)
}

/// Generate project-specific CTO config JSON.
///
/// This creates a config with project-specific values for repository,
/// service name, and other Play workflow settings.
///
/// Uses the shared `cto-config` crate for consistent config generation.
#[must_use]
pub fn generate_project_cto_config(request: &IntakeRequest) -> String {
    use config::{generate_project_config, ProjectConfigInput};

    // Build input for the shared config generator
    let input = ProjectConfigInput {
        repository_url: request.repository_url.clone(),
        project_name: request
            .project_name
            .clone()
            .or_else(|| Some(request.title.clone())),
        team_id: request.team_id.clone(),
        source_branch: request.source_branch.clone(),
        docs_repository: None,
        docs_project_directory: None,
    };

    // Generate config using shared crate
    let config = generate_project_config(&input);

    // Serialize to JSON
    config.to_json().unwrap_or_else(|_| "{}".to_string())
}

/// Derive a service name from a project name (lowercase, hyphenated).
///
/// Re-exports from the shared `cto-config` crate for consistency.
fn derive_service_name(name: &str) -> String {
    config::derive_service_name(name)
}

/// Derive a clean project name from PRD title.
fn derive_project_name(title: &str) -> String {
    // Remove common prefixes
    let cleaned = title
        .trim()
        .strip_prefix("[PRD]")
        .or_else(|| title.strip_prefix("PRD:"))
        .or_else(|| title.strip_prefix("PRD -"))
        .unwrap_or(title)
        .trim();

    // Limit length
    if cleaned.len() > 100 {
        format!("{}...", &cleaned[..97])
    } else {
        cleaned.to_string()
    }
}

/// Format task description for Linear issue.
fn format_task_description(task: &IntakeTask) -> String {
    use std::fmt::Write;

    let mut description = format!("## Description\n\n{}\n\n", task.description);

    if !task.details.is_empty() {
        let _ = write!(description, "## Details\n\n{}\n\n", task.details);
    }

    if !task.test_strategy.is_empty() {
        let _ = write!(
            description,
            "## Test Strategy\n\n{}\n\n",
            task.test_strategy
        );
    }

    if !task.dependencies.is_empty() {
        let deps: Vec<String> = task
            .dependencies
            .iter()
            .map(|d| format!("Task {d}"))
            .collect();
        let _ = write!(
            description,
            "## Dependencies\n\nThis task depends on: {}\n\n",
            deps.join(", ")
        );
    }

    if !task.agent_hint.is_empty() {
        let _ = write!(description, "---\n\n*Agent hint: {}*\n", task.agent_hint);
    }

    description
}

/// Generate intake completion summary.
#[must_use]
#[allow(clippy::implicit_hasher)]
pub fn generate_completion_summary(
    request: &IntakeRequest,
    tasks: &[IntakeTask],
    task_issue_map: &HashMap<String, String>,
) -> String {
    let task_count = tasks.len();
    let high_priority = tasks.iter().filter(|t| t.priority <= 2).count();
    let with_deps = tasks.iter().filter(|t| !t.dependencies.is_empty()).count();
    let issues_created = task_issue_map.len();

    format!(
        r"## ✅ Intake Complete

### Summary
- **PRD**: {}
- **Tasks Generated**: {}
- **High Priority**: {}
- **Tasks with Dependencies**: {}
- **Linear Issues Created**: {}

### Next Steps
1. Review the generated task issues below
2. Adjust priorities or details as needed
3. When ready, delegate a task to `@CTO-Agent` to begin implementation

### Generated Tasks
{}
",
        request.prd_identifier,
        task_count,
        high_priority,
        with_deps,
        issues_created,
        format_task_list(tasks)
    )
}

/// Format task list for summary.
fn format_task_list(tasks: &[IntakeTask]) -> String {
    tasks
        .iter()
        .map(|t| {
            let deps = if t.dependencies.is_empty() {
                String::new()
            } else {
                format!(" (depends on: {:?})", t.dependencies)
            };
            format!(
                "- **Task {}** [P{}]: {}{}\n  {}",
                t.id, t.priority, t.title, deps, t.description
            )
        })
        .collect::<Vec<_>>()
        .join("\n")
}

/// Sanitize project name for Kubernetes resource names.
fn sanitize_project_name(name: &str) -> String {
    name.to_lowercase()
        .chars()
        .map(|c| {
            if c.is_alphanumeric() || c == '-' {
                c
            } else {
                '-'
            }
        })
        .collect::<String>()
        .trim_matches('-')
        .to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_task_description() {
        let task = IntakeTask {
            id: "TASK-001".to_string(),
            title: "Test Task".to_string(),
            description: "Test description".to_string(),
            details: "Implementation details".to_string(),
            dependencies: vec!["TASK-002".to_string(), "TASK-003".to_string()],
            priority: 2,
            test_strategy: "Unit tests".to_string(),
            agent_hint: "rex".to_string(),
        };

        let description = format_task_description(&task);
        assert!(description.contains("Test description"));
        assert!(description.contains("Implementation details"));
        assert!(description.contains("Unit tests"));
        assert!(description.contains("TASK-002"));
        assert!(description.contains("rex"));
    }

    #[test]
    fn test_deserialize_tasks_json_string_ids() {
        let json = r#"{
            "tasks": [
                {
                    "id": "TASK-001",
                    "title": "Setup project",
                    "description": "Initialize the project",
                    "details": "",
                    "dependencies": [],
                    "priority": 1,
                    "testStrategy": "Integration test",
                    "agentHint": "rex"
                }
            ]
        }"#;

        let tasks: TasksJson = serde_json::from_str(json).unwrap();
        assert_eq!(tasks.tasks.len(), 1);
        assert_eq!(tasks.tasks[0].id, "TASK-001");
        assert_eq!(tasks.tasks[0].title, "Setup project");
    }

    #[test]
    fn test_deserialize_tasks_json_numeric_ids() {
        // This is the format that AI agents often generate - numeric IDs instead of strings
        let json = r#"{
            "tasks": [
                {
                    "id": 1,
                    "title": "Setup infrastructure",
                    "description": "Initialize databases and services",
                    "details": "",
                    "dependencies": [],
                    "priority": 1,
                    "testStrategy": "Integration test",
                    "agentHint": "bolt"
                },
                {
                    "id": 2,
                    "title": "Implement backend",
                    "description": "Build the API",
                    "details": "",
                    "dependencies": [1],
                    "priority": 2,
                    "testStrategy": "Unit tests",
                    "agentHint": "rex"
                }
            ]
        }"#;

        let tasks: TasksJson = serde_json::from_str(json).unwrap();
        assert_eq!(tasks.tasks.len(), 2);
        assert_eq!(tasks.tasks[0].id, "1");
        assert_eq!(tasks.tasks[0].title, "Setup infrastructure");
        assert_eq!(tasks.tasks[1].id, "2");
        assert_eq!(tasks.tasks[1].dependencies, vec!["1"]);
    }

    #[test]
    fn test_deserialize_tasks_json_mixed_ids() {
        // Test mixed string and numeric IDs in dependencies
        let json = r#"{
            "tasks": [
                {
                    "id": "1",
                    "title": "Task one",
                    "description": "First task",
                    "dependencies": ["2", 3],
                    "priority": 1
                }
            ]
        }"#;

        let tasks: TasksJson = serde_json::from_str(json).unwrap();
        assert_eq!(tasks.tasks[0].id, "1");
        assert_eq!(tasks.tasks[0].dependencies, vec!["2", "3"]);
    }

    #[test]
    fn test_sanitize_project_name() {
        assert_eq!(sanitize_project_name("TSK-123"), "tsk-123");
        assert_eq!(sanitize_project_name("My Project!"), "my-project");
        assert_eq!(sanitize_project_name("test_name"), "test-name");
    }

    // =========================================================================
    // CTO Config Extraction Tests
    // =========================================================================

    #[test]
    fn test_parse_cto_frontmatter_valid() {
        let description = r"---
cto:
  cli: cursor
  model: claude-opus-4-20250514
---
## PRD: My Feature

This is the actual content.";

        let config = parse_cto_frontmatter(description).unwrap();
        assert_eq!(config.cli, Some("cursor".to_string()));
        assert_eq!(config.model, Some("claude-opus-4-20250514".to_string()));
    }

    #[test]
    fn test_parse_cto_frontmatter_cli_only() {
        let description = r"---
cto:
  cli: codex
---
Content here";

        let config = parse_cto_frontmatter(description).unwrap();
        assert_eq!(config.cli, Some("codex".to_string()));
        assert_eq!(config.model, None);
    }

    #[test]
    fn test_parse_cto_frontmatter_model_only() {
        let description = r"---
cto:
  model: gpt-4.1
---
Content here";

        let config = parse_cto_frontmatter(description).unwrap();
        assert_eq!(config.cli, None);
        assert_eq!(config.model, Some("gpt-4.1".to_string()));
    }

    #[test]
    fn test_parse_cto_frontmatter_no_frontmatter() {
        let description = "Just regular content without frontmatter";
        assert!(parse_cto_frontmatter(description).is_none());
    }

    #[test]
    fn test_parse_cto_frontmatter_no_cto_section() {
        let description = r"---
title: My PRD
author: Someone
---
Content here";

        assert!(parse_cto_frontmatter(description).is_none());
    }

    #[test]
    fn test_parse_cto_frontmatter_empty_cto() {
        let description = r"---
cto:
---
Content here";

        assert!(parse_cto_frontmatter(description).is_none());
    }

    #[test]
    fn test_parse_cto_frontmatter_invalid_yaml() {
        let description = r"---
cto: [invalid yaml here
---
Content";

        assert!(parse_cto_frontmatter(description).is_none());
    }

    #[test]
    fn test_strip_frontmatter() {
        let description = r"---
cto:
  cli: cursor
---
## My PRD

Content here";

        let stripped = strip_frontmatter(description);
        assert_eq!(stripped, "## My PRD\n\nContent here");
    }

    #[test]
    fn test_strip_frontmatter_no_frontmatter() {
        let description = "Just regular content";
        let stripped = strip_frontmatter(description);
        assert_eq!(stripped, "Just regular content");
    }

    #[test]
    fn test_extract_config_from_labels_grouped() {
        let labels = vec![
            Label {
                id: "1".to_string(),
                name: "CTO CLI/cursor".to_string(),
                color: None,
            },
            Label {
                id: "2".to_string(),
                name: "CTO Model/opus".to_string(),
                color: None,
            },
        ];

        let config = extract_config_from_labels(&labels);
        assert_eq!(config.cli, Some("cursor".to_string()));
        assert_eq!(config.model, Some("claude-opus-4-5-20251101".to_string()));
    }

    #[test]
    fn test_extract_config_from_labels_flat() {
        let labels = vec![
            Label {
                id: "1".to_string(),
                name: "claude".to_string(),
                color: None,
            },
            Label {
                id: "2".to_string(),
                name: "sonnet".to_string(),
                color: None,
            },
        ];

        let config = extract_config_from_labels(&labels);
        assert_eq!(config.cli, Some("claude".to_string()));
        assert_eq!(config.model, Some("claude-sonnet-4-5-20250929".to_string()));
    }

    #[test]
    fn test_extract_config_from_labels_mixed() {
        // Mixed: grouped CLI label + arbitrary model label (not in known shortcuts)
        let labels = vec![
            Label {
                id: "1".to_string(),
                name: "CTO CLI/codex".to_string(),
                color: None,
            },
            Label {
                id: "2".to_string(),
                name: "CTO Model/gpt-4.1".to_string(), // Full model name in group label
                color: None,
            },
        ];

        let config = extract_config_from_labels(&labels);
        assert_eq!(config.cli, Some("codex".to_string()));
        assert_eq!(config.model, Some("gpt-4.1".to_string()));
    }

    #[test]
    fn test_extract_config_from_labels_empty() {
        let labels: Vec<Label> = vec![];
        let config = extract_config_from_labels(&labels);
        assert!(config.is_empty());
    }

    #[test]
    fn test_extract_config_from_labels_unrelated() {
        let labels = vec![
            Label {
                id: "1".to_string(),
                name: "bug".to_string(),
                color: None,
            },
            Label {
                id: "2".to_string(),
                name: "high-priority".to_string(),
                color: None,
            },
        ];

        let config = extract_config_from_labels(&labels);
        assert!(config.is_empty());
    }

    #[test]
    fn test_extract_config_from_labels_full_model_in_group() {
        // If someone uses a full model name in the group label
        let labels = vec![Label {
            id: "1".to_string(),
            name: "CTO Model/claude-sonnet-4-20250514".to_string(),
            color: None,
        }];

        let config = extract_config_from_labels(&labels);
        assert_eq!(config.model, Some("claude-sonnet-4-20250514".to_string()));
    }

    // =========================================================================
    // Priority Deserialization Tests
    // =========================================================================

    #[test]
    fn test_deserialize_priority_string_critical() {
        let json = r#"{
            "id": "1",
            "title": "Test Task",
            "description": "Test",
            "priority": "critical"
        }"#;

        let task: IntakeTask = serde_json::from_str(json).unwrap();
        assert_eq!(task.priority, 1); // critical = urgent = 1
    }

    #[test]
    fn test_deserialize_priority_string_high() {
        let json = r#"{
            "id": "1",
            "title": "Test Task",
            "description": "Test",
            "priority": "high"
        }"#;

        let task: IntakeTask = serde_json::from_str(json).unwrap();
        assert_eq!(task.priority, 2); // high = 2
    }

    #[test]
    fn test_deserialize_priority_string_medium() {
        let json = r#"{
            "id": "1",
            "title": "Test Task",
            "description": "Test",
            "priority": "medium"
        }"#;

        let task: IntakeTask = serde_json::from_str(json).unwrap();
        assert_eq!(task.priority, 3); // medium = normal = 3
    }

    #[test]
    fn test_deserialize_priority_string_low() {
        let json = r#"{
            "id": "1",
            "title": "Test Task",
            "description": "Test",
            "priority": "low"
        }"#;

        let task: IntakeTask = serde_json::from_str(json).unwrap();
        assert_eq!(task.priority, 4); // low = 4
    }

    #[test]
    fn test_deserialize_priority_integer() {
        let json = r#"{
            "id": "1",
            "title": "Test Task",
            "description": "Test",
            "priority": 2
        }"#;

        let task: IntakeTask = serde_json::from_str(json).unwrap();
        assert_eq!(task.priority, 2); // Integer passed through
    }

    #[test]
    fn test_deserialize_priority_default() {
        let json = r#"{
            "id": "1",
            "title": "Test Task",
            "description": "Test"
        }"#;

        let task: IntakeTask = serde_json::from_str(json).unwrap();
        assert_eq!(task.priority, 3); // Default = normal = 3
    }

    #[test]
    fn test_deserialize_priority_string_urgent() {
        let json = r#"{
            "id": "1",
            "title": "Test Task",
            "description": "Test",
            "priority": "urgent"
        }"#;

        let task: IntakeTask = serde_json::from_str(json).unwrap();
        assert_eq!(task.priority, 1); // urgent = 1
    }

    #[test]
    fn test_deserialize_priority_string_normal() {
        let json = r#"{
            "id": "1",
            "title": "Test Task",
            "description": "Test",
            "priority": "normal"
        }"#;

        let task: IntakeTask = serde_json::from_str(json).unwrap();
        assert_eq!(task.priority, 3); // normal = 3
    }

    #[test]
    fn test_deserialize_priority_case_insensitive() {
        let json = r#"{
            "id": "1",
            "title": "Test Task",
            "description": "Test",
            "priority": "HIGH"
        }"#;

        let task: IntakeTask = serde_json::from_str(json).unwrap();
        assert_eq!(task.priority, 2); // HIGH = high = 2
    }

    // =========================================================================
    // Agent Validation Tests
    // =========================================================================

    #[test]
    fn test_validate_agent_assignments_correct() {
        let tasks = vec![
            IntakeTask {
                id: "1".to_string(),
                title: "Provision PostgreSQL".to_string(),
                description: "Deploy database cluster".to_string(),
                details: String::new(),
                dependencies: vec![],
                priority: 1,
                test_strategy: String::new(),
                agent_hint: "bolt".to_string(),
            },
            IntakeTask {
                id: "2".to_string(),
                title: "Implement API".to_string(),
                description: "Build REST endpoints".to_string(),
                details: String::new(),
                dependencies: vec!["1".to_string()],
                priority: 2,
                test_strategy: String::new(),
                agent_hint: "rex".to_string(),
            },
        ];

        let result = validate_agent_assignments(&tasks);
        assert!(result.warnings.is_empty());
        assert_eq!(result.support_agent_implementation_count, 0);
    }

    #[test]
    fn test_validate_agent_assignments_cipher_for_implementation() {
        let tasks = vec![IntakeTask {
            id: "1".to_string(),
            title: "JWT Authentication".to_string(),
            description: "Implement JWT middleware".to_string(),
            details: String::new(),
            dependencies: vec![],
            priority: 2,
            test_strategy: String::new(),
            agent_hint: "cipher".to_string(), // Wrong! Should be implementation agent
        }];

        let result = validate_agent_assignments(&tasks);
        assert_eq!(result.warnings.len(), 1);
        assert!(result.warnings[0].contains("JWT Authentication"));
        assert!(result.warnings[0].contains("cipher"));
        assert!(result.warnings[0].contains("implementation task"));
    }

    #[test]
    fn test_validate_agent_assignments_cipher_for_audit_ok() {
        let tasks = vec![IntakeTask {
            id: "1".to_string(),
            title: "Security Audit".to_string(),
            description: "Review authentication implementation for vulnerabilities".to_string(),
            details: String::new(),
            dependencies: vec![],
            priority: 2,
            test_strategy: String::new(),
            agent_hint: "cipher".to_string(), // Correct for security audit
        }];

        let result = validate_agent_assignments(&tasks);
        assert!(result.warnings.is_empty());
    }

    #[test]
    fn test_validate_agent_assignments_tess_for_implementation() {
        let tasks = vec![IntakeTask {
            id: "1".to_string(),
            title: "Rate Limiting Service".to_string(),
            description: "Implement rate limiting middleware".to_string(),
            details: String::new(),
            dependencies: vec![],
            priority: 2,
            test_strategy: String::new(),
            agent_hint: "tess".to_string(), // Wrong! Should be implementation agent
        }];

        let result = validate_agent_assignments(&tasks);
        assert_eq!(result.warnings.len(), 1);
        assert!(result.warnings[0].contains("Rate Limiting"));
    }

    #[test]
    fn test_validate_agent_assignments_tess_for_testing_ok() {
        let tasks = vec![IntakeTask {
            id: "1".to_string(),
            title: "Write Test Suite".to_string(),
            description: "E2E testing for API endpoints".to_string(),
            details: String::new(),
            dependencies: vec![],
            priority: 3,
            test_strategy: String::new(),
            agent_hint: "tess".to_string(), // Correct for testing
        }];

        let result = validate_agent_assignments(&tasks);
        assert!(result.warnings.is_empty());
    }

    #[test]
    fn test_validate_agent_assignments_multiple_errors() {
        let tasks = vec![
            IntakeTask {
                id: "1".to_string(),
                title: "OAuth2 Token Management".to_string(),
                description: "Implement OAuth2 flow".to_string(),
                details: String::new(),
                dependencies: vec![],
                priority: 2,
                test_strategy: String::new(),
                agent_hint: "cipher".to_string(),
            },
            IntakeTask {
                id: "2".to_string(),
                title: "RBAC Middleware".to_string(),
                description: "Role-based access control".to_string(),
                details: String::new(),
                dependencies: vec![],
                priority: 2,
                test_strategy: String::new(),
                agent_hint: "cipher".to_string(),
            },
            IntakeTask {
                id: "3".to_string(),
                title: "Analytics Dashboard".to_string(),
                description: "Build analytics UI".to_string(),
                details: String::new(),
                dependencies: vec![],
                priority: 3,
                test_strategy: String::new(),
                agent_hint: "cleo".to_string(),
            },
        ];

        let result = validate_agent_assignments(&tasks);
        assert_eq!(result.support_agent_implementation_count, 3);
        // Should have summary warning + 3 individual warnings
        assert!(result.warnings.len() >= 3);
        assert!(result.warnings[0].contains("3 tasks"));
    }

    // =========================================================================
    // CTO Config Generation Tests
    // =========================================================================

    /// Helper to create a test `IntakeRequest` with minimal required fields
    fn create_test_intake_request() -> IntakeRequest {
        IntakeRequest {
            session_id: "test-session-123".to_string(),
            access_token: None,
            prd_issue_id: "issue-456".to_string(),
            prd_identifier: "TSK-1".to_string(),
            team_id: "team-789".to_string(),
            title: "My Test Project".to_string(),
            project_name: Some("my-test-project".to_string()),
            prd_content: "Test PRD content".to_string(),
            architecture_content: None,
            repository_url: Some("https://github.com/5dlabs/my-test-project".to_string()),
            github_visibility: "private".to_string(),
            source_branch: Some("main".to_string()),
            tech_stack: TechStack::default(),
            cto_config: crate::config::CtoConfig::default(),
            existing_project: None,
        }
    }

    #[test]
    fn test_generate_project_cto_config_valid_json() {
        let request = create_test_intake_request();
        let config_str = generate_project_cto_config(&request);

        // Should produce valid JSON
        let parsed: serde_json::Value =
            serde_json::from_str(&config_str).expect("Generated config should be valid JSON");

        // Should have version
        assert_eq!(parsed["version"], "1.0");

        // Should have defaults section
        assert!(parsed["defaults"].is_object());
        assert!(parsed["defaults"]["intake"].is_object());
        assert!(parsed["defaults"]["linear"].is_object());
        assert!(parsed["defaults"]["play"].is_object());

        // Should have agents section
        assert!(parsed["agents"].is_object());
    }

    #[test]
    fn test_generate_project_cto_config_repository_extraction() {
        let mut request = create_test_intake_request();
        request.repository_url = Some("https://github.com/myorg/myrepo".to_string());

        let config_str = generate_project_cto_config(&request);
        let parsed: serde_json::Value = serde_json::from_str(&config_str).unwrap();

        // Repository should be extracted without https://github.com/ prefix
        assert_eq!(parsed["defaults"]["play"]["repository"], "myorg/myrepo");
        assert_eq!(parsed["defaults"]["play"]["docsRepository"], "myorg/myrepo");
    }

    #[test]
    fn test_generate_project_cto_config_repository_with_git_suffix() {
        let mut request = create_test_intake_request();
        request.repository_url = Some("https://github.com/myorg/myrepo.git".to_string());

        let config_str = generate_project_cto_config(&request);
        let parsed: serde_json::Value = serde_json::from_str(&config_str).unwrap();

        // Should strip .git suffix
        assert_eq!(parsed["defaults"]["play"]["repository"], "myorg/myrepo");
    }

    #[test]
    fn test_generate_project_cto_config_ssh_repository() {
        let mut request = create_test_intake_request();
        request.repository_url = Some("git@github.com:myorg/myrepo.git".to_string());

        let config_str = generate_project_cto_config(&request);
        let parsed: serde_json::Value = serde_json::from_str(&config_str).unwrap();

        // Should handle SSH-style URLs
        assert_eq!(parsed["defaults"]["play"]["repository"], "myorg/myrepo");
    }

    #[test]
    fn test_generate_project_cto_config_no_repository() {
        let mut request = create_test_intake_request();
        request.repository_url = None;

        let config_str = generate_project_cto_config(&request);
        let parsed: serde_json::Value = serde_json::from_str(&config_str).unwrap();

        // Should fall back to empty string when no repository URL is provided
        // (intake workflow will create a new repository)
        assert_eq!(parsed["defaults"]["play"]["repository"], "");
    }

    #[test]
    fn test_generate_project_cto_config_service_name() {
        let mut request = create_test_intake_request();
        request.project_name = Some("My Cool Project!".to_string());

        let config_str = generate_project_cto_config(&request);
        let parsed: serde_json::Value = serde_json::from_str(&config_str).unwrap();

        // Service name should be sanitized (lowercase, hyphenated)
        assert_eq!(parsed["defaults"]["play"]["service"], "my-cool-project");
    }

    #[test]
    fn test_generate_project_cto_config_team_id() {
        let mut request = create_test_intake_request();
        request.team_id = "team-abc-123".to_string();

        let config_str = generate_project_cto_config(&request);
        let parsed: serde_json::Value = serde_json::from_str(&config_str).unwrap();

        // Team ID should be included in linear config
        assert_eq!(parsed["defaults"]["linear"]["teamId"], "team-abc-123");
    }

    #[test]
    fn test_generate_project_cto_config_source_branch() {
        let mut request = create_test_intake_request();
        request.source_branch = Some("develop".to_string());

        let config_str = generate_project_cto_config(&request);
        let parsed: serde_json::Value = serde_json::from_str(&config_str).unwrap();

        // Source branch should be included
        assert_eq!(parsed["defaults"]["intake"]["sourceBranch"], "develop");
    }

    #[test]
    fn test_generate_project_cto_config_default_source_branch() {
        let mut request = create_test_intake_request();
        request.source_branch = None;

        let config_str = generate_project_cto_config(&request);
        let parsed: serde_json::Value = serde_json::from_str(&config_str).unwrap();

        // Should default to "main"
        assert_eq!(parsed["defaults"]["intake"]["sourceBranch"], "main");
    }

    #[test]
    fn test_generate_project_cto_config_all_agents_present() {
        let request = create_test_intake_request();
        let config_str = generate_project_cto_config(&request);
        let parsed: serde_json::Value = serde_json::from_str(&config_str).unwrap();

        // All expected agents should be present
        let expected_agents = [
            "morgan", "rex", "blaze", "cleo", "tess", "cipher", "atlas", "bolt",
        ];

        for agent in expected_agents {
            assert!(
                parsed["agents"][agent].is_object(),
                "Agent '{agent}' should be present in config"
            );

            // Each agent should have required fields
            assert!(
                parsed["agents"][agent]["githubApp"].is_string(),
                "Agent '{agent}' should have githubApp"
            );
            assert!(
                parsed["agents"][agent]["cli"].is_string(),
                "Agent '{agent}' should have cli"
            );
            assert!(
                parsed["agents"][agent]["model"].is_string(),
                "Agent '{agent}' should have model"
            );
            assert!(
                parsed["agents"][agent]["tools"].is_object(),
                "Agent '{agent}' should have tools"
            );
        }
    }

    #[test]
    fn test_generate_project_cto_config_agent_tools() {
        let request = create_test_intake_request();
        let config_str = generate_project_cto_config(&request);
        let parsed: serde_json::Value = serde_json::from_str(&config_str).unwrap();

        // Rex should have GitHub tools
        let rex_tools = &parsed["agents"]["rex"]["tools"]["remote"];
        assert!(rex_tools.is_array());
        let rex_tools_arr: Vec<&str> = rex_tools
            .as_array()
            .unwrap()
            .iter()
            .map(|v| v.as_str().unwrap())
            .collect();
        assert!(rex_tools_arr.contains(&"github_create_pull_request"));
        assert!(rex_tools_arr.contains(&"github_push_files"));

        // Blaze should have shadcn tools
        let blaze_tools = &parsed["agents"]["blaze"]["tools"]["remote"];
        let blaze_tools_arr: Vec<&str> = blaze_tools
            .as_array()
            .unwrap()
            .iter()
            .map(|v| v.as_str().unwrap())
            .collect();
        assert!(blaze_tools_arr.contains(&"shadcn_list_components"));
        assert!(blaze_tools_arr.contains(&"shadcn_get_component"));

        // Cipher should have security scanning tools
        let cipher_tools = &parsed["agents"]["cipher"]["tools"]["remote"];
        let cipher_tools_arr: Vec<&str> = cipher_tools
            .as_array()
            .unwrap()
            .iter()
            .map(|v| v.as_str().unwrap())
            .collect();
        assert!(cipher_tools_arr.contains(&"github_list_code_scanning_alerts"));
        assert!(cipher_tools_arr.contains(&"github_list_secret_scanning_alerts"));
    }

    #[test]
    fn test_derive_service_name() {
        assert_eq!(derive_service_name("My Project"), "my-project");
        assert_eq!(derive_service_name("my-project"), "my-project");
        assert_eq!(derive_service_name("My Cool App!"), "my-cool-app");
        assert_eq!(derive_service_name("TEST_PROJECT_123"), "test-project-123");
        assert_eq!(derive_service_name("  spaces  "), "spaces");
    }
}
