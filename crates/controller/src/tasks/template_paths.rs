//! Shared Handlebars template path constants used across the controller.

// ============================================================================
// Shared partials (CLI-agnostic building blocks)
// ============================================================================

// Bootstrap partials
pub const SHARED_BOOTSTRAP_RUST_ENV: &str = "shared/bootstrap/rust-env.sh.hbs";

// Function partials
pub const SHARED_FUNCTIONS_GITHUB_AUTH: &str = "shared/functions/github-auth.sh.hbs";
pub const SHARED_FUNCTIONS_DOCKER_SIDECAR: &str = "shared/functions/docker-sidecar.sh.hbs";
pub const SHARED_FUNCTIONS_COMPLETION_MARKER: &str = "shared/functions/completion-marker.sh.hbs";

// Prompt partials
pub const SHARED_PROMPTS_CONTEXT7: &str = "shared/context7-instructions.md.hbs";
pub const SHARED_PROMPTS_DESIGN_SYSTEM: &str = "shared/design-system.md";

// ============================================================================
// CLI-specific templates
// ============================================================================

// Claude code templates
pub const CODE_CLAUDE_CONTAINER_TEMPLATE: &str = "code/claude/container.sh.hbs";
pub const CODE_CLAUDE_MEMORY_TEMPLATE: &str = "code/claude/memory.md.hbs";
pub const CODE_CLAUDE_SETTINGS_TEMPLATE: &str = "code/claude/settings.json.hbs";
pub const CODE_CLAUDE_CONFIG_TEMPLATE: &str = "code/claude/config.json.hbs";

// Shared code templates
pub const CODE_MCP_CONFIG_TEMPLATE: &str = "code/mcp.json.hbs";
pub const CODE_CODING_GUIDELINES_TEMPLATE: &str = "code/coding-guidelines.md.hbs";
pub const CODE_GITHUB_GUIDELINES_TEMPLATE: &str = "code/github-guidelines.md.hbs";

// Codex code templates
pub const CODE_CODEX_CONTAINER_BASE_TEMPLATE: &str = "code/codex/container-base.sh.hbs";
pub const CODE_CODEX_CONTAINER_TEMPLATE: &str = "code/codex/container.sh.hbs";
pub const CODE_CODEX_AGENTS_TEMPLATE: &str = "code/codex/agents.md.hbs";
pub const CODE_CODEX_CONFIG_TEMPLATE: &str = "code/codex/config.toml.hbs";

// Cursor code templates
pub const CODE_CURSOR_CONTAINER_BASE_TEMPLATE: &str = "code/cursor/container-base.sh.hbs";
pub const CODE_CURSOR_CONTAINER_TEMPLATE: &str = "code/cursor/container.sh.hbs";
pub const CODE_CURSOR_AGENTS_TEMPLATE: &str = "code/cursor/agents.md.hbs";
pub const CODE_CURSOR_GLOBAL_CONFIG_TEMPLATE: &str = "code/cursor/cursor-cli-config.json.hbs";
pub const CODE_CURSOR_PROJECT_CONFIG_TEMPLATE: &str = "code/cursor/cursor-cli.json.hbs";

// Factory code templates
pub const CODE_FACTORY_CONTAINER_BASE_TEMPLATE: &str = "code/factory/container-base.sh.hbs";
pub const CODE_FACTORY_CONTAINER_TEMPLATE: &str = "code/factory/container.sh.hbs";
pub const CODE_FACTORY_AGENTS_TEMPLATE: &str = "code/factory/agents.md.hbs";
pub const CODE_FACTORY_GLOBAL_CONFIG_TEMPLATE: &str = "code/factory/factory-cli-config.json.hbs";
pub const CODE_FACTORY_PROJECT_CONFIG_TEMPLATE: &str = "code/factory/factory-cli.json.hbs";

// OpenCode code templates
pub const CODE_OPENCODE_CONTAINER_BASE_TEMPLATE: &str = "code/opencode/container-base.sh.hbs";
pub const CODE_OPENCODE_MEMORY_TEMPLATE: &str = "code/opencode/memory.md.hbs";
pub const CODE_OPENCODE_CONFIG_TEMPLATE: &str = "code/opencode/config.json.hbs";

// Gemini code templates
pub const CODE_GEMINI_CONTAINER_BASE_TEMPLATE: &str = "code/gemini/container-base.sh.hbs";
pub const CODE_GEMINI_CONTAINER_TEMPLATE: &str = "code/gemini/container.sh.hbs";
pub const CODE_GEMINI_MEMORY_TEMPLATE: &str = "code/gemini/memory.md.hbs";
pub const CODE_GEMINI_CONFIG_TEMPLATE: &str = "code/gemini/config.json.hbs";

// Review templates (Stitch PR Review)
pub const REVIEW_FACTORY_CONTAINER_TEMPLATE: &str = "review/factory/container.sh.hbs";
pub const REVIEW_FACTORY_AGENTS_TEMPLATE: &str = "review/factory/agents.md.hbs";
pub const REVIEW_FACTORY_POST_REVIEW_TEMPLATE: &str = "review/factory/post_review.py";
pub const REVIEW_CLAUDE_CONTAINER_TEMPLATE: &str = "review/claude/container.sh.hbs";
pub const REVIEW_CLAUDE_AGENTS_TEMPLATE: &str = "review/claude/agents.md.hbs";

// Remediate templates (Rex PR Remediation)
pub const REMEDIATE_FACTORY_CONTAINER_TEMPLATE: &str = "remediate/factory/container.sh.hbs";
pub const REMEDIATE_FACTORY_AGENTS_TEMPLATE: &str = "remediate/factory/agents.md.hbs";
pub const REMEDIATE_CLAUDE_CONTAINER_TEMPLATE: &str = "remediate/claude/container.sh.hbs";
pub const REMEDIATE_CLAUDE_AGENTS_TEMPLATE: &str = "remediate/claude/agents.md.hbs";
