//! Template paths for heal remediation agents.
//!
//! Heal templates are located in `/agent-templates/heal/{cli}/`
//! where {cli} is either "claude" or "factory".

/// Template path constants for heal agents.
pub const HEAL_CLAUDE_AGENTS_TEMPLATE: &str = "/agent-templates/heal/claude/agents.md.hbs";
pub const HEAL_CLAUDE_CONTAINER_TEMPLATE: &str = "/agent-templates/heal/claude/container.sh.hbs";
pub const HEAL_FACTORY_AGENTS_TEMPLATE: &str = "/agent-templates/heal/factory/agents.md.hbs";
pub const HEAL_FACTORY_CONTAINER_TEMPLATE: &str = "/agent-templates/heal/factory/container.sh.hbs";

/// Template path utilities for heal remediation.
pub struct HealTemplatePaths;

impl HealTemplatePaths {
    /// Get the agents.md template path for a given CLI type.
    #[must_use]
    pub fn agents_template(cli_type: &str) -> &'static str {
        match cli_type.to_lowercase().as_str() {
            "factory" => HEAL_FACTORY_AGENTS_TEMPLATE,
            _ => HEAL_CLAUDE_AGENTS_TEMPLATE,
        }
    }

    /// Get the container.sh template path for a given CLI type.
    #[must_use]
    pub fn container_template(cli_type: &str) -> &'static str {
        match cli_type.to_lowercase().as_str() {
            "factory" => HEAL_FACTORY_CONTAINER_TEMPLATE,
            _ => HEAL_CLAUDE_CONTAINER_TEMPLATE,
        }
    }

    /// Get the base template directory for a given CLI type.
    #[must_use]
    pub fn template_dir(cli_type: &str) -> String {
        let cli = match cli_type.to_lowercase().as_str() {
            "factory" => "factory",
            _ => "claude",
        };
        format!("/agent-templates/heal/{cli}")
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
        assert_eq!(
            HealTemplatePaths::template_dir("claude"),
            "/agent-templates/heal/claude"
        );
        assert_eq!(
            HealTemplatePaths::template_dir("factory"),
            "/agent-templates/heal/factory"
        );
    }
}
