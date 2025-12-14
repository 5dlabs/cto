//! Template paths for heal remediation agents.
//!
//! Heal/remediation templates use the agent template structure under `agents/{agent}/healer/`.
//! Templates are accessed via AGENT_TEMPLATES_PATH env var (embedded in Docker image).
//!
//! Note: Most remediation template resolution uses `template_paths::REMEDIATE_*` constants.
//! This module provides utilities for dynamic agent-based lookups.

/// Default remediation agent (rex) template paths.
/// These are relative paths - the base path comes from AGENT_TEMPLATES_PATH env var.
pub const HEAL_CLAUDE_AGENTS_TEMPLATE: &str = "agents/rex/healer/system-prompt.md.hbs";
pub const HEAL_CLAUDE_CONTAINER_TEMPLATE: &str = "agents/rex/healer/container.sh.hbs";
pub const HEAL_FACTORY_AGENTS_TEMPLATE: &str = "agents/rex/healer/system-prompt.md.hbs";
pub const HEAL_FACTORY_CONTAINER_TEMPLATE: &str = "agents/rex/healer/container.sh.hbs";

/// Template path utilities for heal remediation.
pub struct HealTemplatePaths;

impl HealTemplatePaths {
    /// Get the system-prompt template path for a given CLI type.
    /// Returns a relative path to be combined with AGENT_TEMPLATES_PATH.
    #[must_use]
    pub fn agents_template(cli_type: &str) -> &'static str {
        match cli_type.to_lowercase().as_str() {
            "factory" => HEAL_FACTORY_AGENTS_TEMPLATE,
            _ => HEAL_CLAUDE_AGENTS_TEMPLATE,
        }
    }

    /// Get the container.sh template path for a given CLI type.
    /// Returns a relative path to be combined with AGENT_TEMPLATES_PATH.
    #[must_use]
    pub fn container_template(cli_type: &str) -> &'static str {
        match cli_type.to_lowercase().as_str() {
            "factory" => HEAL_FACTORY_CONTAINER_TEMPLATE,
            _ => HEAL_CLAUDE_CONTAINER_TEMPLATE,
        }
    }

    /// Get the template directory for a given agent's healer templates.
    /// Returns a relative path to be combined with AGENT_TEMPLATES_PATH.
    #[must_use]
    pub fn template_dir(agent: &str) -> String {
        format!("agents/{agent}/healer")
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
    fn test_template_dir() {
        assert_eq!(HealTemplatePaths::template_dir("rex"), "agents/rex/healer");
        assert_eq!(
            HealTemplatePaths::template_dir("blaze"),
            "agents/blaze/healer"
        );
    }
}
