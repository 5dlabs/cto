# Task 1.3 – Template Generator & Tests for Cursor

## Dependencies
- Tasks 1.1 and 1.2 (enum + templates available).

## Parallelization Guidance
- Can run in parallel with Group 2, but coordinate if both modify `controller/src/tasks/code/templates.rs` or Helm ConfigMaps.

## Task Prompt
Extend the template generator and test harness so Cursor templates are emitted and validated alongside Codex/Claude.

Steps:
1. `controller/src/tasks/code/templates.rs`
   - Update `determine_cli_type`, `generate_all_templates`, and helper functions to branch on `CLIType::Cursor`.
   - Implement `get_cursor_container_template`, `get_cursor_memory_template`, etc., mirroring Codex helpers but pointing to `code/cursor/...` paths.
   - Ensure Cursor context includes workflow name (`workflow_name`) so the auto-PR fallback can label runs correctly (lesson from Codex bug where labels failed).
2. `controller/src/bin/test-templates.rs`
   - Register Cursor partial (`CODE_CURSOR_CONTAINER_BASE_TEMPLATE`) and add render calls for container, agents memory, config.
   - Add sample data ensuring `approval_policy` renders as `never`, `sandbox_mode` as `danger-full-access`, and `reasoningEffort` plumbed through.
3. Tests:
   - Extend existing unit tests in `controller/src/tasks/code/templates.rs` to cover Cursor template selection (e.g., `test_cursor_agent_template_selection`).
   - Add assertions verifying `cursor-config.toml` contains the `approval_policy = "never"` line and the Toolman MCP block.
4. Ensure `make`/`cargo test` run the new coverage (add modules to `#[cfg(test)]` blocks as needed).

## Acceptance Criteria
- Running `cargo run --manifest-path controller/Cargo.toml --bin test-templates` outputs Cursor templates without error.
- Unit tests fail if `approval_policy` or `sandbox_mode` deviate (guardrails for future regressions).
- No duplicated code between Cursor and Codex helpers—shared logic should be factored to avoid drift (e.g., reuse partial registration loops where possible).

## Implementation Notes / References
- Watch for panic risk: generator should fall back gracefully if a Cursor agent lacks role-specific template (default to generic container like Codex implementation).
- Maintain consistent naming for constants in `controller/src/tasks/template_paths.rs` (add Cursor entries).
- After this task, `helm template` should still succeed even if Cursor assets aren’t referenced yet; Group 2 will wire the ConfigMap pieces.
