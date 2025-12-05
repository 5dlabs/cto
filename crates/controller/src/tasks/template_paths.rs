//! Shared Handlebars template path constants used across the controller.
//!
//! All templates are located in `/agent-templates/` (mounted from ConfigMap).
//! Structure:
//!   - `_shared/` - Shared partials and container base
//!   - `clis/{cli}/` - CLI-specific config templates
//!   - `{agent}/{job}/` - Agent + job type templates

// ============================================================================
// Shared partials (CLI-agnostic building blocks)
// ============================================================================

// Main container template (uses partials)
pub const SHARED_CONTAINER_TEMPLATE: &str = "_shared/container.sh.hbs";

// Partials directory
pub const SHARED_PARTIALS_DIR: &str = "_shared/partials";

// Individual partials (for explicit registration)
pub const PARTIAL_HEADER: &str = "_shared/partials/header.sh.hbs";
pub const PARTIAL_RUST_ENV: &str = "_shared/partials/rust-env.sh.hbs";
pub const PARTIAL_GO_ENV: &str = "_shared/partials/go-env.sh.hbs";
pub const PARTIAL_NODE_ENV: &str = "_shared/partials/node-env.sh.hbs";
pub const PARTIAL_EXPO_ENV: &str = "_shared/partials/expo-env.sh.hbs";
pub const PARTIAL_CONFIG: &str = "_shared/partials/config.sh.hbs";
pub const PARTIAL_GITHUB_AUTH: &str = "_shared/partials/github-auth.sh.hbs";
pub const PARTIAL_GIT_SETUP: &str = "_shared/partials/git-setup.sh.hbs";
pub const PARTIAL_TASK_FILES: &str = "_shared/partials/task-files.sh.hbs";
pub const PARTIAL_ACCEPTANCE_PROBE: &str = "_shared/partials/acceptance-probe.sh.hbs";
pub const PARTIAL_RETRY_LOOP: &str = "_shared/partials/retry-loop.sh.hbs";
pub const PARTIAL_COMPLETION: &str = "_shared/partials/completion.sh.hbs";

// ============================================================================
// CLI-specific templates (config files, invocation scripts)
// ============================================================================

// Claude CLI
pub const CLI_CLAUDE_CONFIG: &str = "clis/claude/config.json.hbs";
pub const CLI_CLAUDE_SETTINGS: &str = "clis/claude/settings.json.hbs";
pub const CLI_CLAUDE_INVOKE: &str = "clis/claude/invoke.sh.hbs";

// Codex CLI
pub const CLI_CODEX_CONFIG: &str = "clis/codex/config.toml.hbs";
pub const CLI_CODEX_INVOKE: &str = "clis/codex/invoke.sh.hbs";

// Cursor CLI
pub const CLI_CURSOR_CONFIG: &str = "clis/cursor/config.json.hbs";
pub const CLI_CURSOR_MCP: &str = "clis/cursor/mcp.json.hbs";
pub const CLI_CURSOR_INVOKE: &str = "clis/cursor/invoke.sh.hbs";

// Factory CLI
pub const CLI_FACTORY_CONFIG: &str = "clis/factory/config.json.hbs";
pub const CLI_FACTORY_INVOKE: &str = "clis/factory/invoke.sh.hbs";

// Gemini CLI
pub const CLI_GEMINI_CONFIG: &str = "clis/gemini/config.json.hbs";
pub const CLI_GEMINI_SETTINGS: &str = "clis/gemini/settings.json.hbs";
pub const CLI_GEMINI_INVOKE: &str = "clis/gemini/invoke.sh.hbs";

// OpenCode CLI
pub const CLI_OPENCODE_CONFIG: &str = "clis/opencode/config.json.hbs";
pub const CLI_OPENCODE_INVOKE: &str = "clis/opencode/invoke.sh.hbs";

// ============================================================================
// Agent identity templates
// ============================================================================

pub const AGENT_IDENTITY_TEMPLATE: &str = "agents/{agent}/identity.md.hbs";
pub const AGENT_TOOLS_TEMPLATE: &str = "agents/{agent}/tools.hbs";

// ============================================================================
// Job-type templates (agent + job combinations)
// ============================================================================

/// Get container template path for an agent/job combination
/// Falls back to shared container if agent-specific doesn't exist
pub fn agent_container_template(agent: &str, job: &str) -> String {
    format!("{agent}/{job}/container.sh.hbs")
}

/// Get system prompt path for an agent/job combination
pub fn agent_system_prompt(agent: &str, job: &str) -> String {
    format!("{agent}/{job}/system-prompt.md.hbs")
}

// ============================================================================
// Legacy compatibility paths (mapped to new structure)
// These are kept for gradual migration - can be removed once all code updated
// ============================================================================

// Claude code templates (legacy)
pub const CODE_CLAUDE_CONTAINER_TEMPLATE: &str = "_shared/container.sh.hbs";
pub const CODE_CLAUDE_MEMORY_TEMPLATE: &str = "legacy/code/claude/memory.md.hbs";
pub const CODE_CLAUDE_SETTINGS_TEMPLATE: &str = "clis/claude/settings.json.hbs";
pub const CODE_CLAUDE_CONFIG_TEMPLATE: &str = "clis/claude/config.json.hbs";

// Shared code templates (legacy)
pub const CODE_MCP_CONFIG_TEMPLATE: &str = "legacy/code/mcp.json.hbs";
pub const CODE_CODING_GUIDELINES_TEMPLATE: &str = "legacy/code/coding-guidelines.md.hbs";
pub const CODE_GITHUB_GUIDELINES_TEMPLATE: &str = "legacy/code/github-guidelines.md.hbs";

// Codex code templates (legacy)
pub const CODE_CODEX_CONTAINER_BASE_TEMPLATE: &str = "_shared/container.sh.hbs";
pub const CODE_CODEX_CONTAINER_TEMPLATE: &str = "_shared/container.sh.hbs";
pub const CODE_CODEX_AGENTS_TEMPLATE: &str = "legacy/code/codex/agents.md.hbs";
pub const CODE_CODEX_CONFIG_TEMPLATE: &str = "clis/codex/config.toml.hbs";

// Cursor code templates (legacy)
pub const CODE_CURSOR_CONTAINER_BASE_TEMPLATE: &str = "_shared/container.sh.hbs";
pub const CODE_CURSOR_CONTAINER_TEMPLATE: &str = "_shared/container.sh.hbs";
pub const CODE_CURSOR_AGENTS_TEMPLATE: &str = "legacy/code/cursor/agents.md.hbs";
pub const CODE_CURSOR_GLOBAL_CONFIG_TEMPLATE: &str = "clis/cursor/config.json.hbs";
pub const CODE_CURSOR_PROJECT_CONFIG_TEMPLATE: &str = "clis/cursor/mcp.json.hbs";

// Factory code templates (legacy)
pub const CODE_FACTORY_CONTAINER_BASE_TEMPLATE: &str = "_shared/container.sh.hbs";
pub const CODE_FACTORY_CONTAINER_TEMPLATE: &str = "_shared/container.sh.hbs";
pub const CODE_FACTORY_AGENTS_TEMPLATE: &str = "legacy/code/factory/agents.md.hbs";
pub const CODE_FACTORY_GLOBAL_CONFIG_TEMPLATE: &str = "clis/factory/config.json.hbs";
pub const CODE_FACTORY_PROJECT_CONFIG_TEMPLATE: &str = "clis/factory/config.json.hbs";

// OpenCode code templates (legacy)
pub const CODE_OPENCODE_CONTAINER_BASE_TEMPLATE: &str = "_shared/container.sh.hbs";
pub const CODE_OPENCODE_MEMORY_TEMPLATE: &str = "legacy/code/opencode/memory.md.hbs";
pub const CODE_OPENCODE_CONFIG_TEMPLATE: &str = "clis/opencode/config.json.hbs";

// Gemini code templates (legacy)
pub const CODE_GEMINI_CONTAINER_BASE_TEMPLATE: &str = "_shared/container.sh.hbs";
pub const CODE_GEMINI_CONTAINER_TEMPLATE: &str = "_shared/container.sh.hbs";
pub const CODE_GEMINI_MEMORY_TEMPLATE: &str = "legacy/code/gemini/memory.md.hbs";
pub const CODE_GEMINI_CONFIG_TEMPLATE: &str = "clis/gemini/config.json.hbs";

// Review templates (Stitch PR Review) - legacy
pub const REVIEW_FACTORY_CONTAINER_TEMPLATE: &str = "stitch/review/container.sh.hbs";
pub const REVIEW_FACTORY_AGENTS_TEMPLATE: &str = "legacy/review/factory/agents.md.hbs";
pub const REVIEW_FACTORY_POST_REVIEW_TEMPLATE: &str = "legacy/review/factory/post_review.py";
pub const REVIEW_CLAUDE_CONTAINER_TEMPLATE: &str = "stitch/review/container.sh.hbs";
pub const REVIEW_CLAUDE_AGENTS_TEMPLATE: &str = "legacy/review/claude/agents.md.hbs";

// Remediate templates (Rex PR Remediation) - legacy
pub const REMEDIATE_FACTORY_CONTAINER_TEMPLATE: &str = "rex/healer/container.sh.hbs";
pub const REMEDIATE_FACTORY_AGENTS_TEMPLATE: &str = "legacy/remediate/factory/agents.md.hbs";
pub const REMEDIATE_CLAUDE_CONTAINER_TEMPLATE: &str = "rex/healer/container.sh.hbs";
pub const REMEDIATE_CLAUDE_AGENTS_TEMPLATE: &str = "legacy/remediate/claude/agents.md.hbs";

// Shared partials (legacy - mapped to new locations)
pub const SHARED_BOOTSTRAP_RUST_ENV: &str = "_shared/partials/rust-env.sh.hbs";
pub const SHARED_FUNCTIONS_GITHUB_AUTH: &str = "_shared/partials/github-auth.sh.hbs";
pub const SHARED_FUNCTIONS_DOCKER_SIDECAR: &str = "legacy/shared/functions/docker-sidecar.sh.hbs";
pub const SHARED_FUNCTIONS_COMPLETION_MARKER: &str = "_shared/partials/completion.sh.hbs";
pub const SHARED_FUNCTIONS_GIT_OPERATIONS: &str = "_shared/partials/git-setup.sh.hbs";
pub const SHARED_FUNCTIONS_GH_CLI: &str = "legacy/shared/functions/gh-cli.sh.hbs";
pub const SHARED_FUNCTIONS_QUALITY_GATES: &str = "legacy/shared/functions/quality-gates.sh.hbs";
pub const SHARED_PROMPTS_CONTEXT7: &str = "legacy/shared/context7-instructions.md.hbs";
pub const SHARED_PROMPTS_DESIGN_SYSTEM: &str = "legacy/shared/design-system.md";
pub const SHARED_CONTAINER_CORE: &str = "_shared/container.sh.hbs";
