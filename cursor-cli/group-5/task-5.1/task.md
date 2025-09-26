# Task 5.1 – Validation Test Matrix

## Dependencies
- Completion of engineering tasks Groups 1–3 and documentation readiness (Group 4 optionally parallel for doc references).

## Parallelization Guidance
- Must precede staging rollout (Task 5.2).

## Task Prompt
Execute and record the full validation suite ensuring Cursor integration is production-ready.

Checklist:
1. Local/CI commands:
   - `cargo fmt --manifest-path controller/Cargo.toml -- --check`
   - `cargo clippy --manifest-path controller/Cargo.toml --all-targets -- -D warnings`
   - `cargo test --manifest-path controller/Cargo.toml`
   - `cargo fmt --manifest-path mcp/Cargo.toml -- --check` (if MCP touched)
   - `make -C infra/gitops validate`
   - `helm lint infra/charts/controller`
   - `cargo run --manifest-path controller/Cargo.toml --bin test-templates`
2. Dry-run job execution (if Cursor credits available later): run controller locally with fake `CURSOR_API_KEY` to ensure job spec renders and command attempts to start (expect auth failure but parse logs for correct command shape).
3. Capture outputs/logs and store in `Cursor CLI/group-5/task-5.1/validation-log.md` (include command output snippets or references to CI runs).

## Acceptance Criteria
- All commands above succeed (or, for dry-run, fail only due to missing real API key with log evidence that command invocation is correct).
- Validation log checked into repo summarising results and linking to CI runs (GitHub Actions build URLs, etc.).
- Issues found are either resolved or ticketed before proceeding to staging.

## Implementation Notes / References
- Reuse lessons from Codex validation: watch for yamllint warnings (document which ones are tolerated).
- If Cursor CLI requires actual credits for smoke test, note that in log and schedule once funds restored.
