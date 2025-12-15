//! Shared Handlebars template path constants used across the controller.
//!
//! Templates are embedded in the Docker image and accessed via AGENT_TEMPLATES_PATH env var.
//! Structure:
//!   - `_shared/` - Shared partials and container base
//!   - `clis/{cli}/` - CLI-specific config templates
//!   - `agents/{agent}/{job}/` - Agent + job type templates

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
pub const PARTIAL_TOOLS_CONFIG: &str = "_shared/partials/tools-config.sh.hbs";
pub const PARTIAL_ACCEPTANCE_PROBE: &str = "_shared/partials/acceptance-probe.sh.hbs";
pub const PARTIAL_RETRY_LOOP: &str = "_shared/partials/retry-loop.sh.hbs";
pub const PARTIAL_COMPLETION: &str = "_shared/partials/completion.sh.hbs";

// Frontend stack partials (for Blaze agent)
pub const PARTIAL_FRONTEND_TOOLKITS: &str = "_shared/partials/frontend-toolkits.md.hbs";
pub const PARTIAL_TANSTACK_STACK: &str = "_shared/partials/tanstack-stack.md.hbs";
pub const PARTIAL_SHADCN_STACK: &str = "_shared/partials/shadcn-stack.md.hbs";

// Infrastructure operators partial (for Bolt/Morgan agents)
pub const PARTIAL_INFRASTRUCTURE_OPERATORS: &str =
    "_shared/partials/infrastructure-operators.md.hbs";

// ============================================================================
// CLI-specific templates (invocation scripts, settings)
// Config templates removed - adapters now serialize directly
// ============================================================================

// Claude CLI
pub const CLI_CLAUDE_INVOKE: &str = "clis/claude/invoke.sh.hbs";

// Codex CLI
pub const CLI_CODEX_INVOKE: &str = "clis/codex/invoke.sh.hbs";

// Cursor CLI
pub const CLI_CURSOR_INVOKE: &str = "clis/cursor/invoke.sh.hbs";

// Factory CLI
pub const CLI_FACTORY_INVOKE: &str = "clis/factory/invoke.sh.hbs";

// Gemini CLI
pub const CLI_GEMINI_INVOKE: &str = "clis/gemini/invoke.sh.hbs";

// OpenCode CLI
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
#[must_use]
pub fn agent_container_template(agent: &str, job: &str) -> String {
    format!("{agent}/{job}/container.sh.hbs")
}

/// Get system prompt path for an agent/job combination
#[must_use]
pub fn agent_system_prompt(agent: &str, job: &str) -> String {
    format!("{agent}/{job}/system-prompt.md.hbs")
}

// ============================================================================
// Container templates (all now use shared container)
// ============================================================================

pub const CODE_CLAUDE_CONTAINER_TEMPLATE: &str = "_shared/container.sh.hbs";
pub const CODE_CODEX_CONTAINER_BASE_TEMPLATE: &str = "_shared/container.sh.hbs";
pub const CODE_CODEX_CONTAINER_TEMPLATE: &str = "_shared/container.sh.hbs";
pub const CODE_CURSOR_CONTAINER_BASE_TEMPLATE: &str = "_shared/container.sh.hbs";
pub const CODE_CURSOR_CONTAINER_TEMPLATE: &str = "_shared/container.sh.hbs";
pub const CODE_FACTORY_CONTAINER_BASE_TEMPLATE: &str = "_shared/container.sh.hbs";
pub const CODE_FACTORY_CONTAINER_TEMPLATE: &str = "_shared/container.sh.hbs";
pub const CODE_OPENCODE_CONTAINER_BASE_TEMPLATE: &str = "_shared/container.sh.hbs";
pub const CODE_GEMINI_CONTAINER_BASE_TEMPLATE: &str = "_shared/container.sh.hbs";
pub const CODE_GEMINI_CONTAINER_TEMPLATE: &str = "_shared/container.sh.hbs";

// Review/Remediate container templates (agent-specific)
pub const REVIEW_FACTORY_CONTAINER_TEMPLATE: &str = "agents/stitch/review/container.sh.hbs";
pub const REVIEW_CLAUDE_CONTAINER_TEMPLATE: &str = "agents/stitch/review/container.sh.hbs";
pub const REMEDIATE_FACTORY_CONTAINER_TEMPLATE: &str = "agents/rex/healer/container.sh.hbs";
pub const REMEDIATE_CLAUDE_CONTAINER_TEMPLATE: &str = "agents/rex/healer/container.sh.hbs";

// ============================================================================
// Memory/System-prompt templates (default fallbacks for health checks)
// Production code uses CodeTemplateGenerator.get_agent_system_prompt_template()
// ============================================================================

pub const CODE_CLAUDE_MEMORY_TEMPLATE: &str = "agents/rex/coder/system-prompt.md.hbs";
pub const CODE_CODEX_AGENTS_TEMPLATE: &str = "agents/rex/coder/system-prompt.md.hbs";
pub const CODE_CURSOR_AGENTS_TEMPLATE: &str = "agents/rex/coder/system-prompt.md.hbs";
pub const CODE_FACTORY_AGENTS_TEMPLATE: &str = "agents/rex/coder/system-prompt.md.hbs";
pub const CODE_OPENCODE_MEMORY_TEMPLATE: &str = "agents/rex/coder/system-prompt.md.hbs";
pub const CODE_GEMINI_MEMORY_TEMPLATE: &str = "agents/rex/coder/system-prompt.md.hbs";
pub const REVIEW_FACTORY_AGENTS_TEMPLATE: &str = "agents/stitch/review/system-prompt.md.hbs";
pub const REVIEW_CLAUDE_AGENTS_TEMPLATE: &str = "agents/stitch/review/system-prompt.md.hbs";
pub const REMEDIATE_FACTORY_AGENTS_TEMPLATE: &str = "agents/rex/healer/system-prompt.md.hbs";
pub const REMEDIATE_CLAUDE_AGENTS_TEMPLATE: &str = "agents/rex/healer/system-prompt.md.hbs";

// ============================================================================
// Shared code templates
// ============================================================================

pub const CODE_MCP_CONFIG_TEMPLATE: &str = "_shared/partials/tools-config.sh.hbs";
pub const CODE_CODING_GUIDELINES_TEMPLATE: &str = "agents/rex/coder/system-prompt.md.hbs";
pub const CODE_GITHUB_GUIDELINES_TEMPLATE: &str = "agents/rex/coder/system-prompt.md.hbs";
pub const REVIEW_FACTORY_POST_REVIEW_TEMPLATE: &str = "agents/stitch/review/system-prompt.md.hbs";

// ============================================================================
// Shared partials (legacy names mapped to new locations)
// Note: These are kept for backwards compatibility with older templates.
// The docker-sidecar and gh-cli functions no longer exist as separate partials;
// their functionality is integrated into the shared container template.
// ============================================================================

pub const SHARED_BOOTSTRAP_RUST_ENV: &str = "_shared/partials/rust-env.sh.hbs";
pub const SHARED_FUNCTIONS_GITHUB_AUTH: &str = "_shared/partials/github-auth.sh.hbs";
pub const SHARED_FUNCTIONS_COMPLETION_MARKER: &str = "_shared/partials/completion.sh.hbs";
pub const SHARED_FUNCTIONS_GIT_OPERATIONS: &str = "_shared/partials/git-setup.sh.hbs";
pub const SHARED_FUNCTIONS_QUALITY_GATES: &str = "_shared/partials/acceptance-probe.sh.hbs";
pub const SHARED_PROMPTS_CONTEXT7: &str = "agents/rex/coder/system-prompt.md.hbs";
pub const SHARED_PROMPTS_DESIGN_SYSTEM: &str = "agents/blaze/coder/system-prompt.md.hbs";
pub const SHARED_CONTAINER_CORE: &str = "_shared/container.sh.hbs";
