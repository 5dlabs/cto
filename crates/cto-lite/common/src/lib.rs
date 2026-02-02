//! Shared utilities for CTO Lite components

use anyhow::{anyhow, Result};

/// Sanitize a string for use in Kubernetes resource names (DNS-1123 compliant)
pub fn sanitize_k8s_name(s: &str, max_len: usize) -> String {
    s.to_lowercase()
        .replace(['/', '_'], "-")
        .chars()
        .filter(|c| c.is_ascii_alphanumeric() || *c == '-')
        .take(max_len)
        .collect::<String>()
        .trim_end_matches('-')
        .trim_start_matches('-')
        .to_string()
}

/// Validate repository format (owner/repo)
pub fn validate_repo(repo: &str) -> Result<()> {
    if repo.is_empty() {
        return Err(anyhow!("Repository name cannot be empty"));
    }
    if !repo.contains('/') || repo.split('/').count() != 2 {
        return Err(anyhow!(
            "Invalid repository format. Expected 'owner/repo', got '{repo}'"
        ));
    }
    if repo.len() > 200 {
        return Err(anyhow!("Repository name too long (max 200 chars)"));
    }
    // Check for valid characters (alphanumeric, hyphen, underscore, dot, slash)
    if !repo
        .chars()
        .all(|c| c.is_alphanumeric() || c == '-' || c == '_' || c == '.' || c == '/')
    {
        return Err(anyhow!(
            "Repository name contains invalid characters. Only alphanumeric, hyphen, underscore, dot allowed"
        ));
    }
    Ok(())
}

/// Validate stack parameter
pub fn validate_stack(stack: &str) -> Result<()> {
    match stack {
        "nova" | "grizz" => Ok(()),
        _ => Err(anyhow!(
            "Invalid stack '{stack}'. Must be 'nova' or 'grizz'"
        )),
    }
}

/// Validate prompt length
pub fn validate_prompt(prompt: &str) -> Result<()> {
    const MAX_PROMPT_LEN: usize = 50_000;
    if prompt.is_empty() {
        return Err(anyhow!("Prompt cannot be empty"));
    }
    if prompt.len() > MAX_PROMPT_LEN {
        return Err(anyhow!(
            "Prompt too long ({} chars, max {} chars)",
            prompt.len(),
            MAX_PROMPT_LEN
        ));
    }
    Ok(())
}
