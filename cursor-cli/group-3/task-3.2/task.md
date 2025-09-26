# Task 3.2 – Job Spec & Volume Mount Integration

## Dependencies
- Task 3.1 (adapter) and Group 2 tasks (values/secrets/configmaps).

## Parallelization Guidance
- Work closely with Task 3.3 to ensure policies are testable once job spec changes land.

## Task Prompt
Ensure the controller-generated Kubernetes Job mounts Cursor templates, exposes secrets, and configures environment variables so the adapter/template combo runs successfully.

Tasks:
1. `controller/src/tasks/code/resources.rs`
   - When `CLIType::Cursor`, mount `code/cursor/**` templates into `/task-files` (mirror codex branch).
   - Inject env vars: `CURSOR_API_KEY`, `CURSOR_CONFIG_PATH` if needed, `TOOLMAN_SERVER_URL`.
   - Confirm volume names remain unique; consider reusing shared `agent-templates` volume.
2. ConfigMap keys
   - Update whichever generator writes the static templates (after Group 2) to include Cursor paths; ensure mount logic references correct keys.
3. Resource naming
   - Update naming utilities if they embed CLI type (e.g., `resources.rs::select_image_for_cli`, `naming.rs`). Cursor should select the correct image and produce deterministic PVC names.
4. Tests
   - Extend existing unit tests (e.g., `select_image_for_cli` tests) to cover Cursor case.
   - Add fixture-based test verifying Job JSON includes `/task-files/cursor-config.toml`, `CURSOR_API_KEY`, `cursor-agent` command.

## Acceptance Criteria
- Running `cargo test --package controller tasks::code::resources` passes with Cursor coverage.
- Generated Job spec (inspect via unit test or debug log) shows proper mounts/env; compare with Codex job to confirm parity.
- No regressions for Claude/Codex job specs (existing tests continue to pass).

## Implementation Notes / References
- Reuse new CLI config merge logic we added for Codex reasoning effort to avoid duplication.
- Ensure `approvalPolicy` is enforced via env/config rather than job spec flags; keep responsibilities split (templates handle CLI flagging, job spec handles secrets & mounts).
- Remember to update `controller/src/tasks/config.rs` if any defaults need Cursor-specific overrides.
