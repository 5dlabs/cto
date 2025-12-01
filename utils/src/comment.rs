//! # PR Comments
//!
//! Create and manage comments on GitHub pull requests using the `gh` CLI.
//!
//! ## Example
//!
//! ```no_run
//! use utils::PrComment;
//!
//! # async fn example() -> anyhow::Result<()> {
//! let comment = PrComment::new("5dlabs", "cto");
//!
//! // Post a simple comment
//! comment.post(1864, "Hello from utils!").await?;
//! # Ok(())
//! # }
//! ```

use anyhow::{Context, Result};
use std::collections::HashMap;
use std::process::Stdio;
use tokio::process::Command;
use tracing::{debug, info, warn};

use crate::alerts::{Annotation, AnnotationLevel};

/// Number of context lines to show around the error
const CONTEXT_LINES: u32 = 2;

/// Client for posting comments to GitHub PRs
#[derive(Debug, Clone)]
pub struct PrComment {
    owner: String,
    repo: String,
}

impl PrComment {
    /// Create a new PR comment client
    pub fn new(owner: impl Into<String>, repo: impl Into<String>) -> Self {
        Self {
            owner: owner.into(),
            repo: repo.into(),
        }
    }

    /// Post a comment to a PR
    pub async fn post(&self, pr_number: u32, body: &str) -> Result<()> {
        debug!(pr = pr_number, body_len = body.len(), "Posting PR comment");

        let output = Command::new("gh")
            .args([
                "pr",
                "comment",
                &pr_number.to_string(),
                "--repo",
                &format!("{}/{}", self.owner, self.repo),
                "--body",
                body,
            ])
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .output()
            .await
            .context("Failed to execute gh CLI")?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            anyhow::bail!("gh pr comment failed: {stderr}");
        }

        info!(pr = pr_number, "Posted comment to PR");
        Ok(())
    }

    /// Post a formatted alerts summary to a PR with code snippets
    pub async fn post_alerts_with_context(
        &self,
        pr_number: u32,
        sha: &str,
        annotations: &[Annotation],
    ) -> Result<()> {
        // Fetch file contents for annotations
        let file_cache = self.fetch_files_for_annotations(sha, annotations).await;
        let body = format_alerts_comment_with_context(annotations, &file_cache);
        self.post(pr_number, &body).await
    }

    /// Fetch file contents for all annotations
    async fn fetch_files_for_annotations(
        &self,
        sha: &str,
        annotations: &[Annotation],
    ) -> HashMap<String, Vec<String>> {
        let mut cache: HashMap<String, Vec<String>> = HashMap::new();

        // Get unique file paths (skip .github pseudo-paths)
        let paths: Vec<_> = annotations
            .iter()
            .map(|a| a.path.as_str())
            .filter(|p| !p.starts_with(".github") && !p.is_empty())
            .collect::<std::collections::HashSet<_>>()
            .into_iter()
            .collect();

        for path in paths {
            match self.fetch_file_content(sha, path).await {
                Ok(content) => {
                    let lines: Vec<String> = content.lines().map(String::from).collect();
                    cache.insert(path.to_string(), lines);
                }
                Err(e) => {
                    warn!(path, error = %e, "Failed to fetch file content");
                }
            }
        }

        cache
    }

    /// Fetch a single file's content from the repo at a specific SHA
    async fn fetch_file_content(&self, sha: &str, path: &str) -> Result<String> {
        let output = Command::new("gh")
            .args([
                "api",
                &format!(
                    "repos/{}/{}/contents/{}?ref={sha}",
                    self.owner, self.repo, path
                ),
                "--jq",
                ".content",
            ])
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .output()
            .await
            .context("Failed to execute gh api")?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            anyhow::bail!("Failed to fetch {path}: {stderr}");
        }

        // GitHub returns base64-encoded content
        let base64_content = String::from_utf8_lossy(&output.stdout).trim().to_string();
        // Remove any quotes if present
        let base64_content = base64_content.trim_matches('"');

        // Decode base64 (content may have newlines)
        let decoded = base64_decode(base64_content)?;
        Ok(decoded)
    }
}

/// Decode base64 content (handles line breaks in GitHub's response)
fn base64_decode(input: &str) -> Result<String> {
    // Remove newlines that GitHub adds
    let clean: String = input.chars().filter(|c| !c.is_whitespace()).collect();

    // Use a simple base64 decode
    let bytes = base64_decode_bytes(&clean)?;
    String::from_utf8(bytes).context("File content is not valid UTF-8")
}

/// Simple base64 decoder
fn base64_decode_bytes(input: &str) -> Result<Vec<u8>> {
    const ALPHABET: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";

    let mut output = Vec::with_capacity(input.len() * 3 / 4);
    let mut buffer: u32 = 0;
    let mut bits_collected = 0;

    for c in input.bytes() {
        if c == b'=' {
            break;
        }
        let value = ALPHABET
            .iter()
            .position(|&x| x == c)
            .context("Invalid base64 character")?;

        buffer = (buffer << 6) | (value as u32);
        bits_collected += 6;

        if bits_collected >= 8 {
            bits_collected -= 8;
            output.push((buffer >> bits_collected) as u8);
            buffer &= (1 << bits_collected) - 1;
        }
    }

    Ok(output)
}

/// Format annotations as a markdown comment with code snippets
pub fn format_alerts_comment_with_context(
    annotations: &[Annotation],
    file_cache: &HashMap<String, Vec<String>>,
) -> String {
    if annotations.is_empty() {
        return "## ✅ No CI Failures\n\nAll checks passed without errors.".to_string();
    }

    let mut md = String::new();

    // Header
    md.push_str("## 🚨 CI Alerts Summary\n\n");

    // Stats with severity badges
    let failure_count = annotations
        .iter()
        .filter(|a| a.level == AnnotationLevel::Failure)
        .count();
    let warning_count = annotations
        .iter()
        .filter(|a| a.level == AnnotationLevel::Warning)
        .count();
    let notice_count = annotations
        .iter()
        .filter(|a| a.level == AnnotationLevel::Notice)
        .count();

    let mut stats = Vec::new();
    if failure_count > 0 {
        stats.push(format!("🔴 **{failure_count}** error(s)"));
    }
    if warning_count > 0 {
        stats.push(format!("🟡 **{warning_count}** warning(s)"));
    }
    if notice_count > 0 {
        stats.push(format!("🔵 **{notice_count}** notice(s)"));
    }
    md.push_str(&stats.join(" · "));
    md.push_str("\n\n");

    // Failures first
    let failures: Vec<_> = annotations
        .iter()
        .filter(|a| a.level == AnnotationLevel::Failure)
        .filter(|a| !a.path.starts_with(".github")) // Skip workflow meta-errors
        .collect();

    if !failures.is_empty() {
        md.push_str("### ❌ Errors\n\n");
        for ann in failures {
            md.push_str(&format_annotation_with_context(ann, file_cache));
        }
    }

    // Then warnings
    let warnings: Vec<_> = annotations
        .iter()
        .filter(|a| a.level == AnnotationLevel::Warning)
        .filter(|a| !a.path.starts_with(".github"))
        .collect();

    if !warnings.is_empty() {
        md.push_str("### ⚠️ Warnings\n\n");
        for ann in warnings {
            md.push_str(&format_annotation_with_context(ann, file_cache));
        }
    }

    // Then notices
    let notices: Vec<_> = annotations
        .iter()
        .filter(|a| a.level == AnnotationLevel::Notice)
        .filter(|a| !a.path.starts_with(".github"))
        .collect();

    if !notices.is_empty() {
        md.push_str("### ℹ️ Notices\n\n");
        for ann in notices {
            md.push_str(&format_annotation_with_context(ann, file_cache));
        }
    }

    // Footer
    md.push_str("\n---\n<sub>Generated by `utils post-alerts`</sub>\n");

    md
}

/// Format a single annotation with code context
fn format_annotation_with_context(
    ann: &Annotation,
    file_cache: &HashMap<String, Vec<String>>,
) -> String {
    use std::fmt::Write;

    let mut s = String::new();

    // Severity badge
    let badge = match ann.level {
        AnnotationLevel::Failure => "🔴",
        AnnotationLevel::Warning => "🟡",
        AnnotationLevel::Notice => "🔵",
    };

    // Extract lint name if present (e.g., "clippy::needless_return")
    let (lint_name, lint_link) = extract_lint_info(&ann.message);

    // File location header
    let line_info = if ann.start_line == ann.end_line {
        format!("L{}", ann.start_line)
    } else {
        format!("L{}-{}", ann.start_line, ann.end_line)
    };

    let _ = write!(s, "<details>\n<summary>{badge} ");
    let _ = write!(s, "<code>{}</code>:{}", ann.path, ann.start_line);

    // Add lint name if found
    if let Some(name) = &lint_name {
        if let Some(link) = &lint_link {
            let _ = write!(s, " — <a href=\"{link}\"><code>{name}</code></a>");
        } else {
            let _ = write!(s, " — <code>{name}</code>");
        }
    }
    let _ = writeln!(s, "</summary>\n");

    // Message
    let _ = writeln!(s, "**{}**\n", ann.message);

    // Code snippet if available
    if let Some(lines) = file_cache.get(&ann.path) {
        if let Some(snippet) = extract_code_snippet(lines, ann.start_line, ann.end_line) {
            // Determine language for syntax highlighting
            let lang = get_language_for_file(&ann.path);
            let _ = writeln!(s, "```{lang}");
            let _ = write!(s, "{snippet}");
            let _ = writeln!(s, "```");
        }
    }

    // Link to file on GitHub
    let _ = writeln!(
        s,
        "\n[View in repository]({line_info})\n",
    );

    let _ = writeln!(s, "</details>\n");

    s
}

/// Extract lint name and documentation link from message
fn extract_lint_info(message: &str) -> (Option<String>, Option<String>) {
    // Look for clippy lint patterns like "clippy::needless_return" or just lint names
    let clippy_pattern = regex::Regex::new(r"clippy::(\w+)").ok();

    if let Some(re) = clippy_pattern {
        if let Some(caps) = re.captures(message) {
            let lint_name = caps.get(1).map(|m| m.as_str().to_string());
            if let Some(ref name) = lint_name {
                let link =
                    format!("https://rust-lang.github.io/rust-clippy/master/index.html#{name}");
                return (Some(format!("clippy::{name}")), Some(link));
            }
        }
    }

    // Try to extract lint name from common patterns
    let known_lints = [
        ("unused variable", "unused_variables"),
        ("unused import", "unused_imports"),
        ("dead code", "dead_code"),
        ("needless `return`", "needless_return"),
        ("unneeded `return`", "needless_return"),
        ("useless use of `format!`", "useless_format"),
        ("manual implementation of `Option::map`", "manual_map"),
        ("length comparison to zero", "len_zero"),
        ("writing `&Vec`", "ptr_arg"),
        ("variables can be used directly", "uninlined_format_args"),
    ];

    for (pattern, lint) in known_lints {
        if message.to_lowercase().contains(&pattern.to_lowercase()) {
            let link =
                format!("https://rust-lang.github.io/rust-clippy/master/index.html#{lint}");
            return (Some(format!("clippy::{lint}")), Some(link));
        }
    }

    (None, None)
}

/// Extract code snippet with context lines
fn extract_code_snippet(lines: &[String], start: u32, end: u32) -> Option<String> {
    if lines.is_empty() || start == 0 {
        return None;
    }

    let start_idx = start.saturating_sub(1) as usize; // Convert to 0-indexed
    let end_idx = end.saturating_sub(1) as usize;

    if start_idx >= lines.len() {
        return None;
    }

    // Calculate context range
    let ctx_start = start_idx.saturating_sub(CONTEXT_LINES as usize);
    let ctx_end = (end_idx + CONTEXT_LINES as usize + 1).min(lines.len());

    let mut snippet = String::new();
    for (i, line) in lines[ctx_start..ctx_end].iter().enumerate() {
        let line_num = ctx_start + i + 1; // 1-indexed for display
        let marker = if line_num >= start as usize && line_num <= end as usize {
            ">" // Mark the error line
        } else {
            " "
        };
        use std::fmt::Write;
        let _ = writeln!(snippet, "{line_num:>4} {marker}{line}");
    }

    Some(snippet)
}

/// Get syntax highlighting language for a file
fn get_language_for_file(path: &str) -> &'static str {
    use std::path::Path;

    let ext = Path::new(path)
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("");

    match ext.to_lowercase().as_str() {
        "rs" => "rust",
        "ts" | "tsx" => "typescript",
        "js" | "jsx" => "javascript",
        "py" => "python",
        "go" => "go",
        "yaml" | "yml" => "yaml",
        "json" => "json",
        "md" => "markdown",
        "sh" | "bash" => "bash",
        _ => "",
    }
}

/// Format annotations as a markdown comment (simple version without code)
pub fn format_alerts_comment(annotations: &[Annotation]) -> String {
    format_alerts_comment_with_context(annotations, &HashMap::new())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_empty_annotations() {
        let result = format_alerts_comment(&[]);
        assert!(result.contains("No CI Failures"));
    }

    #[test]
    fn test_format_single_failure() {
        let annotations = vec![Annotation {
            path: "src/main.rs".to_string(),
            start_line: 10,
            end_line: 10,
            level: AnnotationLevel::Failure,
            message: "unused variable".to_string(),
            title: String::new(),
            raw_details: String::new(),
        }];

        let result = format_alerts_comment(&annotations);
        assert!(result.contains("CI Alerts Summary"));
        assert!(result.contains("1** error"));
        assert!(result.contains("src/main.rs"));
        assert!(result.contains("unused variable"));
    }

    #[test]
    fn test_format_mixed_levels() {
        let annotations = vec![
            Annotation {
                path: "src/lib.rs".to_string(),
                start_line: 5,
                end_line: 5,
                level: AnnotationLevel::Failure,
                message: "error message".to_string(),
                title: String::new(),
                raw_details: String::new(),
            },
            Annotation {
                path: "src/lib.rs".to_string(),
                start_line: 10,
                end_line: 10,
                level: AnnotationLevel::Warning,
                message: "warning message".to_string(),
                title: String::new(),
                raw_details: String::new(),
            },
        ];

        let result = format_alerts_comment(&annotations);
        assert!(result.contains("Errors"));
        assert!(result.contains("Warnings"));
        assert!(result.contains("error message"));
        assert!(result.contains("warning message"));
    }

    #[test]
    fn test_extract_lint_info() {
        let (name, link) = extract_lint_info("unused variable: `x`");
        assert_eq!(name, Some("clippy::unused_variables".to_string()));
        assert!(link.is_some());

        let (name, link) = extract_lint_info("unneeded `return` statement");
        assert_eq!(name, Some("clippy::needless_return".to_string()));
        assert!(link.is_some());
    }

    #[test]
    fn test_extract_code_snippet() {
        let lines: Vec<String> = (1..=10).map(|i| format!("line {i}")).collect();

        let snippet = extract_code_snippet(&lines, 5, 5).unwrap();
        assert!(snippet.contains("line 5"));
        assert!(snippet.contains(">line 5")); // Error marker
    }

    #[test]
    fn test_base64_decode() {
        let encoded = "SGVsbG8gV29ybGQ="; // "Hello World"
        let decoded = base64_decode(encoded).unwrap();
        assert_eq!(decoded, "Hello World");
    }

    #[test]
    fn test_pr_comment_new() {
        let comment = PrComment::new("owner", "repo");
        assert_eq!(comment.owner, "owner");
        assert_eq!(comment.repo, "repo");
    }
}
