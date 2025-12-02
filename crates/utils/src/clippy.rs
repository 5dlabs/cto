//! # Clippy Error Parser
//!
//! Fetches and parses Clippy errors from GitHub CI check runs.
//! Provides structured output for automated remediation.
//!
//! ## Example
//!
//! ```no_run
//! use utils::ClippyErrors;
//!
//! # async fn example() -> anyhow::Result<()> {
//! let client = ClippyErrors::new("5dlabs", "cto");
//! let errors = client.fetch(1956).await?;
//!
//! for err in errors {
//!     println!("{}: {} at {}:{}", err.code, err.message, err.file, err.line);
//!     if let Some(suggestion) = &err.suggestion {
//!         println!("  Suggestion: {}", suggestion);
//!     }
//! }
//! # Ok(())
//! # }
//! ```

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::process::Stdio;
use tokio::process::Command;
use tracing::{debug, info};

/// A parsed Clippy error with structured information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClippyError {
    /// File path (e.g., `controller/src/tasks/code/templates.rs`)
    pub file: String,

    /// Line number
    pub line: u32,

    /// Column number (if available)
    pub column: Option<u32>,

    /// Error level: "error" or "warning"
    pub level: String,

    /// Clippy lint code (e.g., `clippy::too_many_lines`)
    pub code: String,

    /// The error message
    pub message: String,

    /// Clippy's suggestion (from "help: try:" lines)
    pub suggestion: Option<String>,

    /// The raw annotation message for context
    pub raw_message: String,
}

/// Client for fetching Clippy errors from GitHub
#[derive(Debug, Clone)]
pub struct ClippyErrors {
    owner: String,
    repo: String,
}

impl ClippyErrors {
    /// Create a new Clippy errors client
    pub fn new(owner: impl Into<String>, repo: impl Into<String>) -> Self {
        Self {
            owner: owner.into(),
            repo: repo.into(),
        }
    }

    /// Fetch Clippy errors for a PR from the lint-rust check run
    pub async fn fetch(&self, pr_number: u32) -> Result<Vec<ClippyError>> {
        let head_sha = self.get_head_sha(pr_number).await?;
        info!(pr = pr_number, sha = %head_sha, "Fetching Clippy errors");

        // Find the lint-rust check run
        let check_run_id = self.find_lint_rust_check_run(&head_sha).await?;

        if let Some(id) = check_run_id {
            let annotations = self.get_annotations(id).await?;
            let errors = self.parse_annotations(annotations);
            info!(pr = pr_number, count = errors.len(), "Found Clippy errors");
            Ok(errors)
        } else {
            debug!(pr = pr_number, "No lint-rust check run found");
            Ok(Vec::new())
        }
    }

    /// Fetch Clippy errors from a specific check run ID
    pub async fn fetch_by_check_run(&self, check_run_id: u64) -> Result<Vec<ClippyError>> {
        let annotations = self.get_annotations(check_run_id).await?;
        Ok(self.parse_annotations(annotations))
    }

    /// Get the head SHA for a PR
    async fn get_head_sha(&self, pr_number: u32) -> Result<String> {
        let output = Command::new("gh")
            .args([
                "pr",
                "view",
                &pr_number.to_string(),
                "--repo",
                &format!("{}/{}", self.owner, self.repo),
                "--json",
                "headRefOid",
                "-q",
                ".headRefOid",
            ])
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .output()
            .await
            .context("Failed to execute gh CLI")?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            anyhow::bail!("gh pr view failed: {stderr}");
        }

        let sha = String::from_utf8_lossy(&output.stdout).trim().to_string();
        Ok(sha)
    }

    /// Find the lint-rust check run for a commit
    async fn find_lint_rust_check_run(&self, sha: &str) -> Result<Option<u64>> {
        let output = Command::new("gh")
            .args([
                "api",
                &format!(
                    "repos/{}/{}/commits/{sha}/check-runs",
                    self.owner, self.repo
                ),
            ])
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .output()
            .await
            .context("Failed to execute gh api")?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            anyhow::bail!("gh api check-runs failed: {stderr}");
        }

        let stdout = String::from_utf8_lossy(&output.stdout);
        let response: CheckRunsResponse =
            serde_json::from_str(&stdout).context("Failed to parse check runs")?;

        // Look for lint-rust check run
        for run in response.check_runs {
            if run.name == "lint-rust" && run.conclusion == Some("failure".to_string()) {
                debug!(check_run_id = run.id, "Found failed lint-rust check run");
                return Ok(Some(run.id));
            }
        }

        Ok(None)
    }

    /// Get annotations for a check run
    async fn get_annotations(&self, check_run_id: u64) -> Result<Vec<RawAnnotation>> {
        let output = Command::new("gh")
            .args([
                "api",
                &format!(
                    "repos/{}/{}/check-runs/{check_run_id}/annotations",
                    self.owner, self.repo
                ),
            ])
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .output()
            .await
            .context("Failed to execute gh api")?;

        if !output.status.success() {
            return Ok(Vec::new());
        }

        let stdout = String::from_utf8_lossy(&output.stdout);
        let annotations: Vec<RawAnnotation> =
            serde_json::from_str(&stdout).context("Failed to parse annotations")?;

        Ok(annotations)
    }

    /// Parse raw annotations into structured Clippy errors
    fn parse_annotations(&self, annotations: Vec<RawAnnotation>) -> Vec<ClippyError> {
        annotations
            .into_iter()
            .filter_map(|ann| self.parse_annotation(ann))
            .collect()
    }

    /// Parse a single annotation into a ClippyError
    fn parse_annotation(&self, ann: RawAnnotation) -> Option<ClippyError> {
        let message = &ann.message;

        // Extract Clippy lint code from message
        // Format: "error: this function has too many lines (114/100)"
        // Or from title: "clippy::too_many_lines"
        let code = self.extract_lint_code(message, &ann.title);

        // Extract suggestion if present
        let suggestion = self.extract_suggestion(message);

        // Determine level from annotation_level
        let level = match ann.annotation_level.as_str() {
            "failure" => "error",
            "warning" => "warning",
            _ => "notice",
        }
        .to_string();

        Some(ClippyError {
            file: ann.path,
            line: ann.start_line,
            column: None,
            level,
            code,
            message: self.extract_main_message(message),
            suggestion,
            raw_message: ann.message,
        })
    }

    /// Extract the Clippy lint code from the message
    fn extract_lint_code(&self, message: &str, title: &str) -> String {
        // Check title first (often contains the lint name)
        if title.starts_with("clippy::") {
            return title.to_string();
        }

        // Look for clippy:: in the message
        if let Some(start) = message.find("clippy::") {
            let rest = &message[start..];
            let end = rest
                .find(|c: char| c.is_whitespace() || c == '`' || c == ']')
                .unwrap_or(rest.len());
            return rest[..end].to_string();
        }

        // Look for lint name in help URL
        // "https://rust-lang.github.io/rust-clippy/rust-1.91.0/index.html#too_many_lines"
        if let Some(start) = message.find('#') {
            let rest = &message[start + 1..];
            let end = rest
                .find(|c: char| c.is_whitespace() || c == '\n')
                .unwrap_or(rest.len());
            return format!("clippy::{}", &rest[..end]);
        }

        "unknown".to_string()
    }

    /// Extract the main error message (first line)
    fn extract_main_message(&self, message: &str) -> String {
        message.lines().next().unwrap_or(message).trim().to_string()
    }

    /// Extract suggestion from "help: try:" or similar patterns
    fn extract_suggestion(&self, message: &str) -> Option<String> {
        // Look for "help: " lines
        for line in message.lines() {
            let trimmed = line.trim();
            if trimmed.starts_with("help:") {
                return Some(trimmed.to_string());
            }
        }

        // Look for suggestion blocks (lines starting with + or -)
        let mut suggestion_lines = Vec::new();
        let mut in_suggestion = false;

        for line in message.lines() {
            let trimmed = line.trim();
            if trimmed.starts_with('+') || trimmed.starts_with('-') {
                in_suggestion = true;
                suggestion_lines.push(line);
            } else if in_suggestion && !trimmed.is_empty() && !trimmed.starts_with('|') {
                break;
            }
        }

        if !suggestion_lines.is_empty() {
            return Some(suggestion_lines.join("\n"));
        }

        None
    }

    /// Generate a markdown prompt for fixing the errors
    pub fn generate_fix_prompt(&self, errors: &[ClippyError]) -> String {
        use std::fmt::Write;

        let mut prompt = String::from("# Clippy Errors to Fix\n\n");

        for (i, err) in errors.iter().enumerate() {
            let _ = writeln!(prompt, "## Error {}: `{}`\n", i + 1, err.code);
            let _ = writeln!(prompt, "**File:** `{}:{}`\n", err.file, err.line);
            let _ = writeln!(prompt, "**Message:** {}\n", err.message);

            if let Some(suggestion) = &err.suggestion {
                let _ = writeln!(prompt, "**Suggestion:**\n```\n{suggestion}\n```\n");
            }

            prompt.push_str("---\n\n");
        }

        prompt.push_str("## Instructions\n\n");
        prompt.push_str("1. Fix each error following Clippy's suggestion if provided\n");
        prompt
            .push_str("2. Run `cargo clippy --workspace --all-targets -- -D warnings` to verify\n");
        prompt.push_str("3. Only fix Clippy errors, don't refactor unrelated code\n");

        prompt
    }
}

/// Raw annotation from GitHub API
#[derive(Debug, Deserialize)]
struct RawAnnotation {
    path: String,
    start_line: u32,
    #[allow(dead_code)]
    end_line: u32,
    annotation_level: String,
    message: String,
    #[serde(default)]
    title: String,
}

/// Check runs API response
#[derive(Debug, Deserialize)]
struct CheckRunsResponse {
    check_runs: Vec<CheckRunRaw>,
}

/// Raw check run from API
#[derive(Debug, Deserialize)]
struct CheckRunRaw {
    id: u64,
    name: String,
    conclusion: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_lint_code() {
        let client = ClippyErrors::new("owner", "repo");

        // From title
        assert_eq!(
            client.extract_lint_code("some message", "clippy::too_many_lines"),
            "clippy::too_many_lines"
        );

        // From message
        let msg = "for further information visit https://rust-lang.github.io/rust-clippy/rust-1.91.0/index.html#too_many_lines";
        assert_eq!(client.extract_lint_code(msg, ""), "clippy::too_many_lines");
    }

    #[test]
    fn test_extract_suggestion() {
        let client = ClippyErrors::new("owner", "repo");

        let msg = "error: unnested or-patterns\nhelp: nest the patterns\n  |\n- Some(\"a\") | Some(\"b\")\n+ Some(\"a\" | \"b\")";
        let suggestion = client.extract_suggestion(msg);
        assert!(suggestion.is_some());
        assert!(suggestion.unwrap().contains("help:"));
    }

    #[test]
    fn test_generate_fix_prompt() {
        let client = ClippyErrors::new("owner", "repo");
        let errors = vec![ClippyError {
            file: "src/main.rs".to_string(),
            line: 10,
            column: None,
            level: "error".to_string(),
            code: "clippy::too_many_lines".to_string(),
            message: "this function has too many lines".to_string(),
            suggestion: Some("help: consider splitting this function".to_string()),
            raw_message: "full message".to_string(),
        }];

        let prompt = client.generate_fix_prompt(&errors);
        assert!(prompt.contains("clippy::too_many_lines"));
        assert!(prompt.contains("src/main.rs:10"));
    }
}
