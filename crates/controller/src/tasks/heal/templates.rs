//! Template paths for heal remediation agents.
//!
//! Heal/remediation templates use the agent template structure under `agents/{agent}/`.
//! Flat structure: agents/{agent}/{job}.md.hbs for system prompts
//! Templates are accessed via AGENT_TEMPLATES_PATH env var (embedded in Docker image).
//!
//! Note: Most remediation template resolution uses `template_paths::REMEDIATE_*` constants.
//! This module provides utilities for dynamic agent-based lookups.

/// Default remediation agent (rex) template paths.
/// These are relative paths - the base path comes from AGENT_TEMPLATES_PATH env var.
/// Flat structure: agents/{agent}/{job}.md.hbs
pub const HEAL_CLAUDE_AGENTS_TEMPLATE: &str = "agents/rex/healer.md.hbs";
pub const HEAL_CLAUDE_CONTAINER_TEMPLATE: &str = "_shared/container.sh.hbs";
pub const HEAL_FACTORY_AGENTS_TEMPLATE: &str = "agents/rex/healer.md.hbs";
pub const HEAL_FACTORY_CONTAINER_TEMPLATE: &str = "_shared/container.sh.hbs";

/// Template path utilities for heal remediation.
pub struct HealTemplatePaths;

impl HealTemplatePaths {
    /// Get the system-prompt template path for a given CLI type.
    /// Returns a relative path to be combined with AGENT_TEMPLATES_PATH.
    /// Flat structure: agents/{agent}/{job}.md.hbs
    #[must_use]
    pub fn agents_template(cli_type: &str) -> &'static str {
        match cli_type.to_lowercase().as_str() {
            "factory" => HEAL_FACTORY_AGENTS_TEMPLATE,
            _ => HEAL_CLAUDE_AGENTS_TEMPLATE,
        }
    }

    /// Get the container.sh template path for a given CLI type.
    /// Returns a relative path to be combined with AGENT_TEMPLATES_PATH.
    /// All agents use the shared container template.
    #[must_use]
    pub fn container_template(cli_type: &str) -> &'static str {
        match cli_type.to_lowercase().as_str() {
            "factory" => HEAL_FACTORY_CONTAINER_TEMPLATE,
            _ => HEAL_CLAUDE_CONTAINER_TEMPLATE,
        }
    }

    /// Get the system prompt path for a given agent's healer job.
    /// Returns a relative path to be combined with AGENT_TEMPLATES_PATH.
    /// Flat structure: agents/{agent}/healer.md.hbs
    #[must_use]
    pub fn system_prompt(agent: &str) -> String {
        format!("agents/{agent}/healer.md.hbs")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_agents_template_claude() {
        assert_eq!(
            HealTemplatePaths::agents_template("claude"),
            HEAL_CLAUDE_AGENTS_TEMPLATE
        );
        assert_eq!(
            HealTemplatePaths::agents_template("Claude"),
            HEAL_CLAUDE_AGENTS_TEMPLATE
        );
    }

    #[test]
    fn test_agents_template_factory() {
        assert_eq!(
            HealTemplatePaths::agents_template("factory"),
            HEAL_FACTORY_AGENTS_TEMPLATE
        );
        assert_eq!(
            HealTemplatePaths::agents_template("Factory"),
            HEAL_FACTORY_AGENTS_TEMPLATE
        );
    }

    #[test]
    fn test_container_template_claude() {
        assert_eq!(
            HealTemplatePaths::container_template("claude"),
            HEAL_CLAUDE_CONTAINER_TEMPLATE
        );
    }

    #[test]
    fn test_container_template_factory() {
        assert_eq!(
            HealTemplatePaths::container_template("factory"),
            HEAL_FACTORY_CONTAINER_TEMPLATE
        );
    }

    #[test]
    fn test_system_prompt() {
        assert_eq!(
            HealTemplatePaths::system_prompt("rex"),
            "agents/rex/healer.md.hbs"
        );
        assert_eq!(
            HealTemplatePaths::system_prompt("blaze"),
            "agents/blaze/healer.md.hbs"
        );
    }
}
