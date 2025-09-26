# Task 3.3 – Policy Enforcement Tests

## Dependencies
- Tasks 3.1 and 3.2 (adapter + job spec implemented).

## Parallelization Guidance
- Can run in parallel with documentation tasks (Group 4) once runtime behaviour is stabilised.

## Task Prompt
Add automated tests guaranteeing Cursor runs inherit the required security policies (approval + sandbox), preventing future regressions like we saw earlier in Codex.

Scope:
1. Unit tests for config rendering (`controller/src/tasks/code/templates.rs`): assert that `cursor-config.toml` contains `approval_policy = "never"` and `sandbox_mode = "danger-full-access"` for any Cursor agent.
2. Job spec tests verifying env vars propagate policy expectations (e.g., environment variable confirming no approval prompts, if applicable).
3. Adapter tests ensuring CLI arguments do not omit necessary flags (e.g., `--print`, `--output-format`, `--force` when required) and do not include stray approval prompts.
4. Negative tests: craft a `CLIConfig` with explicit `approvalPolicy = "on-request"` and ensure the system overrides or logs + fails per design.

## Acceptance Criteria
- Tests fail if policy values change or disappear (guardrail for future edits).
- CI suite (`cargo test`, `cargo fmt`, `cargo clippy`) remains green with new tests.
- Documentation comment near tests explaining why defaults are critical (reference the Codex regression where approval prompts blocked automation).

## Implementation Notes / References
- Reuse patterns from Codex tests recently added when we fixed reasoning effort propagation.
- Consider using snapshot tests or string matching on rendered templates to keep assertions simple.
- Capture log expectations using `tracing_test` if we log warnings when overriding policies.
