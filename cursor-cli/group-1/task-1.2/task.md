# Task 1.2 – Cursor Template Suite

## Dependencies
- Task 1.1 (`Cursor` CLIType available).

## Parallelization Guidance
- Can run alongside Task 1.3 once base template files exist; coordinate on shared paths under `infra/charts/controller/agent-templates/`.

## Task Prompt
Author the handlebars templates that drive Cursor execution, mirroring the Codex structure but incorporating Cursor-specific CLI behaviours.

Deliverables:
1. Directory layout under `infra/charts/controller/agent-templates/code/cursor/` mirroring the Codex structure while honouring Cursor naming conventions:
   - `container-base.sh.hbs` (shared logic)
   - `container-{rex,cleo,tess,...}.sh.hbs` wrappers that inject banners and role-specific instructions
   - `agents*.md.hbs` (memory/prompt documents per agent)
   - `cursor-cli-config.json.hbs` (global CLI settings rendered into `~/.cursor/cli-config.json`)
   - `cursor-cli.json.hbs` (project permissions rendered into `.cursor/cli.json`)
   - Shared artifacts like `client-config.json.hbs` if the CLI expects config (reuse existing template or add cursor-specific conditionals).
2. Container script specifics:
   - Invoke `cursor-agent` instead of `codex exec`.
   - Support both interactive and headless flows: when running in controller jobs we use `--print` + `--output-format stream-json` to capture progress; ensure `--force` is gated by template variables (e.g., remediation vs implementation agent).
   - Enforce approval policy (`--force` requires we set default config to allow writes; also template should set env var/CLI flag to skip interactive approvals).
   - Export `CURSOR_API_KEY` and respect existing GitHub App auth flow (reusing the GitHub App token bootstrap from Codex so PR creation stays consistent).
   - Reuse MCP bootstrap (copy logic from Codex container to copy `/task-files/mcp.json` and set toolman URLs) noting Cursor shares MCP config with the editor.
   - Integrate the improved auto-PR fallback lessons from Codex (branch verification, optional labels, run label handling). Consider sharing logic via partials if duplication with Codex is large.
3. Memory/agents docs:
   - Update instructions to speak to Cursor CLI features (mention `/compress`, MCP usage, shell mode constraints, 30s timeout).
   - Inject explicit guidance on using `--force` responsibly and to respect permissions from `cli-config.json`.
4. Config templates (`cursor-cli-config.json.hbs`, `cursor-cli.json.hbs`):
   - Enforce non-interactive defaults (`approvalPolicy = "never"`, `sandboxMode = "danger-full-access"`) matching the Codex behaviour.
   - Support optional `reasoningEffort` from CLI settings (same path as Codex: `cli_config.settings.reasoningEffort`).
   - Configure the Toolman MCP server identical to Codex (command `toolman`, args `--url ...`) so both global and project configs are ready for headless execution.

## Acceptance Criteria
- `cargo run --manifest-path controller/Cargo.toml --bin test-templates` renders Cursor templates without panic once Task 1.3 wires them up.
- Shellcheck-style sanity: run `bash -n` on rendered `container-base.sh` (via `test-templates` output) to ensure no syntax errors.
- Template variables documented inline (`{{#if}}` blocks comment describing when they fire) to help future maintenance.
- Handlebars partials named consistently (e.g., `{{> cursor_container_base ...}}`) to mirror Codex structure and ease reuse.

## Implementation Notes / References
- Reuse GitHub auth, token refresh, repo cloning snippets from `infra/charts/controller/agent-templates/code/codex/container-base.sh.hbs`; adjust CLI invocations only where necessary.
- `docs/cursor-cli/headless.md` provides canonical examples for `--print`, `--force`, and output formats—embed relevant notes in comments so operators know why flags exist.
- Remember to update `infra/charts/controller/agent-templates/code/github-guidelines.md.hbs` and similar shared docs if Cursor requires different instructions (e.g., emphasise `cursor-agent` commands for local testing).
