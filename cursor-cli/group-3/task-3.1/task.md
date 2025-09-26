# Task 3.1 – Implement Cursor CLI Adapter

## Dependencies
- Tasks 1.1–1.3 (enum + templates + generator) and Task 2.1 (values).

## Parallelization Guidance
- Coordinate with Task 3.2; adapter must expose command/env info consumed by job spec wiring.

## Task Prompt
Implement the Rust adapter responsible for launching Cursor CLI inside controller jobs.

Requirements:
1. `controller/src/cli/adapters/cursor.rs`
   - Define struct `CursorAdapter` implementing the same traits as `CodexAdapter` (e.g., `CliAdapter`).
   - Command builder should produce something like:
     ```rust
     Command::new("cursor-agent")
         .args(["--print", "--output-format", "stream-json"])
         .arg("--force")? // gated based on task type
         .env("CURSOR_API_KEY", ...)
     ```
   - Inject working directory via `--cd` or ensure container script handles `cd` (align with template logic).
   - Support optional CLI overrides from `CLIConfig.settings` (model, temperature, reasoning effort) just as Codex does.
2. Approval policy:
   - Adapter must refuse to run if config does not enforce `approvalPolicy = "never"` (log warning + continue using default).
3. Tests:
   - Add unit tests in `cursor.rs` verifying command line construction given sample config (use `assert_cmd_args` helpers from Codex tests).
   - Ensure `reasoningEffort` is forwarded to config template (fail test if missing).
4. Logging/telemetry: align with existing `CodexAdapter` instrumentation (span names, debug output).

## Acceptance Criteria
- `cargo test -p controller cli::adapters::cursor::*` passes.
- Integration with template generator works: when `CLIType::Cursor` is selected, adapter returns correct binary/args without panic.
- Approval policy enforcement unit test ensures deviation triggers log + default to `never`.

## Implementation Notes / References
- Use Codex adapter as blueprint but replace binary/flags per Cursor docs (`docs/cursor-cli/headless.md`, `parameters.md`).
- Remember to honour environment variables for MCP if provided (`TOOLMAN_SERVER_URL`).
- Consider future extension for interactive mode (document TODOs where behaviour may differ).
