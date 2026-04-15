//! Shared Handlebars template path constants used across the controller.
//!
//! Templates are embedded in the Docker image and accessed via AGENT_TEMPLATES_PATH env var.
//! Structure:
//!   - `_shared/` - Shared partials and container base
//!   - `clis/` - Unified ACP dispatch partial (openclaw.sh.hbs)
//!   - `harness-agents/` - OpenClaw gateway entrypoint + config templates
//!   - `agents/{agent}/` - Agent templates (flat: {job}.md.hbs, {job}.sh.hbs)

// ============================================================================
// Shared partials (CLI-agnostic building blocks)
// ============================================================================

// Main container template (uses partials)
pub const SHARED_CONTAINER_TEMPLATE: &str = "_shared/container.sh.hbs";

// Lobster base task template (replaces container.sh for Lobster-based flows)
pub const LOBSTER_BASE_TASK_TEMPLATE: &str = "lobster/base-task.lobster.yaml.hbs";

// Harness-agent templates (OpenClaw gateway entrypoint)
pub const HARNESS_OPENCLAW_TEMPLATE: &str = "harness-agents/openclaw.sh.hbs";
pub const HARNESS_OPENCLAW_CONFIG_TEMPLATE: &str = "harness-agents/openclaw-config.json.hbs";
pub const HARNESS_HERMES_TEMPLATE: &str = "harness-agents/hermes.sh.hbs";

// Partials directory
pub const SHARED_PARTIALS_DIR: &str = "_shared/partials";

// Individual partials (for explicit registration)
pub const PARTIAL_HEADER: &str = "_shared/partials/header.sh.hbs";
pub const PARTIAL_RUST_ENV: &str = "_shared/partials/rust-env.sh.hbs";
pub const PARTIAL_GO_ENV: &str = "_shared/partials/go-env.sh.hbs";
pub const PARTIAL_NODE_ENV: &str = "_shared/partials/node-env.sh.hbs";
pub const PARTIAL_EXPO_ENV: &str = "_shared/partials/expo-env.sh.hbs";
pub const PARTIAL_UNITY_ENV: &str = "_shared/partials/unity-env.sh.hbs";
pub const PARTIAL_CONFIG: &str = "_shared/partials/config.sh.hbs";
pub const PARTIAL_GITHUB_AUTH: &str = "_shared/partials/github-auth.sh.hbs";
pub const PARTIAL_GIT_SETUP: &str = "_shared/partials/git-setup.sh.hbs";
pub const PARTIAL_TASK_FILES: &str = "_shared/partials/task-files.sh.hbs";
pub const PARTIAL_TOOLS_CONFIG: &str = "_shared/partials/tools-config.sh.hbs";
pub const PARTIAL_CTO_TOOLS_SETUP: &str = "_shared/partials/cto-tools-setup.sh.hbs";
pub const PARTIAL_ACCEPTANCE_PROBE: &str = "_shared/partials/acceptance-probe.sh.hbs";
pub const PARTIAL_RETRY_LOOP: &str = "_shared/partials/retry-loop.sh.hbs";
pub const PARTIAL_COMPLETION: &str = "_shared/partials/completion.sh.hbs";
pub const PARTIAL_MCP_CHECK: &str = "_shared/partials/mcp-check.sh.hbs";
pub const PARTIAL_SKILLS_SETUP: &str = "_shared/partials/skills-setup.sh.hbs";

// Frontend stack partials (for Blaze agent)
pub const PARTIAL_FRONTEND_TOOLKITS: &str = "_shared/partials/frontend-toolkits.md.hbs";
pub const PARTIAL_TANSTACK_STACK: &str = "_shared/partials/tanstack-stack.md.hbs";
pub const PARTIAL_SHADCN_STACK: &str = "_shared/partials/shadcn-stack.md.hbs";

// Infrastructure partials (for Bolt agent)
pub const PARTIAL_INFRASTRUCTURE_OPERATORS: &str =
    "_shared/partials/infrastructure-operators.md.hbs";
pub const PARTIAL_INFRASTRUCTURE_SETUP: &str = "_shared/partials/infrastructure-setup.sh.hbs";
pub const PARTIAL_INFRASTRUCTURE_VERIFY: &str = "_shared/partials/infrastructure-verify.sh.hbs";

// Auth partials referenced by agent system prompts (Spark/Tap/Blaze etc.)
// Wrapped in {{#unless skills_native}} so only fire for CLIs without native skills (Cursor/Gemini).
pub const PARTIAL_BETTER_AUTH: &str = "_shared/partials/better-auth.md.hbs";
pub const PARTIAL_BETTER_AUTH_ELECTRON: &str = "_shared/partials/better-auth-electron.md.hbs";
pub const PARTIAL_BETTER_AUTH_EXPO: &str = "_shared/partials/better-auth-expo.md.hbs";

// ============================================================================
// CTO Tools (static files for dynamic MCP tool access)
// Copied into agent pods via task-files ConfigMap
// ============================================================================

pub const CTO_TOOLS_CLI: &str = "cto-tools/cto-tools";
pub const CTO_TOOLS_MCP_TS: &str = "cto-tools/mcp.ts";

// ============================================================================
// Skills (modular context loaded just-in-time)
// Structure: skills/{category}/{skill_name}/SKILL.md
// ============================================================================

pub const SKILLS_DIR: &str = "skills";
pub const SKILLS_MAPPINGS: &str = "skills/skill-mappings.yaml";

// ============================================================================
// CLI invocation partial (unified ACP dispatch)
// The `clis/openclaw.sh.hbs` template is registered as the `{{> cli_execute}}`
// partial by `register_cli_invoke_partial()` in templates.rs.
// Individual per-CLI templates are archived in `clis/_archived/`.
// ============================================================================

// ============================================================================
// Agent identity templates
// ============================================================================

pub const AGENT_IDENTITY_TEMPLATE: &str = "agents/{agent}/identity.md.hbs";
pub const AGENT_TOOLS_TEMPLATE: &str = "agents/{agent}/tools.hbs";

// ============================================================================
// Job-type templates (agent + job combinations)
// Flat structure: agents/{agent}/{job}.md.hbs, agents/{agent}/{job}.sh.hbs
// ============================================================================

/// Get container template path for an agent/job combination
/// Falls back to shared container if agent-specific doesn't exist
#[must_use]
pub fn agent_container_template(agent: &str, job: &str) -> String {
    format!("{agent}/{job}.sh.hbs")
}

/// Get system prompt path for an agent/job combination
#[must_use]
pub fn agent_system_prompt(agent: &str, job: &str) -> String {
    format!("{agent}/{job}.md.hbs")
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

// Review/Remediate container templates (all use shared container)
// Agent-specific behavior is handled via job_type conditionals in the shared template
pub const REVIEW_FACTORY_CONTAINER_TEMPLATE: &str = "_shared/container.sh.hbs";
pub const REVIEW_CLAUDE_CONTAINER_TEMPLATE: &str = "_shared/container.sh.hbs";
pub const REMEDIATE_FACTORY_CONTAINER_TEMPLATE: &str = "_shared/container.sh.hbs";
pub const REMEDIATE_CLAUDE_CONTAINER_TEMPLATE: &str = "_shared/container.sh.hbs";

// ============================================================================
// Memory/System-prompt templates (default fallbacks for health checks)
// Production code uses CodeTemplateGenerator.get_agent_system_prompt_template()
// Flat structure: agents/{agent}/{job}.md.hbs
// ============================================================================

pub const CODE_CLAUDE_MEMORY_TEMPLATE: &str = "agents/rex/coder.md.hbs";
pub const CODE_CODEX_AGENTS_TEMPLATE: &str = "agents/rex/coder.md.hbs";
pub const CODE_CURSOR_AGENTS_TEMPLATE: &str = "agents/rex/coder.md.hbs";
pub const CODE_FACTORY_AGENTS_TEMPLATE: &str = "agents/rex/coder.md.hbs";
pub const CODE_OPENCODE_MEMORY_TEMPLATE: &str = "agents/rex/coder.md.hbs";
pub const CODE_GEMINI_MEMORY_TEMPLATE: &str = "agents/rex/coder.md.hbs";
pub const REVIEW_FACTORY_AGENTS_TEMPLATE: &str = "agents/stitch/review.md.hbs";
pub const REVIEW_CLAUDE_AGENTS_TEMPLATE: &str = "agents/stitch/review.md.hbs";
pub const REMEDIATE_FACTORY_AGENTS_TEMPLATE: &str = "agents/rex/healer.md.hbs";
pub const REMEDIATE_CLAUDE_AGENTS_TEMPLATE: &str = "agents/rex/healer.md.hbs";

// ============================================================================
// Shared code templates
// ============================================================================

// Note: MCP config is now generated programmatically in templates.rs
// (CODE_MCP_CONFIG_TEMPLATE removed - was "code/mcp.json.hbs")
pub const CODE_CODING_GUIDELINES_TEMPLATE: &str = "agents/rex/coder.md.hbs";
pub const CODE_GITHUB_GUIDELINES_TEMPLATE: &str = "agents/rex/coder.md.hbs";
pub const REVIEW_FACTORY_POST_REVIEW_TEMPLATE: &str = "agents/stitch/review.md.hbs";

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
pub const SHARED_PROMPTS_CONTEXT7: &str = "agents/rex/coder.md.hbs";
pub const SHARED_PROMPTS_DESIGN_SYSTEM: &str = "agents/blaze/coder.md.hbs";
pub const SHARED_CONTAINER_CORE: &str = "_shared/container.sh.hbs";
