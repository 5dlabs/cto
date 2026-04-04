Implement subtask 8001: Configure Bun test runner and E2E environment bootstrap

## Objective
Set up the Bun test runner for the E2E test suite, configure environment variables for all external service endpoints (cto-pm intake, Linear, GitHub, Discord bridge, Hermes/NOUS), set the global test timeout to 5 minutes, and create shared test utilities (unique run ID generator, HTTP client wrappers, assertion helpers).

## Steps
Step-by-step:
1. Create `tests/e2e/` directory with `pipeline.test.ts` entry file.
2. Configure `bunfig.toml` or test config with 5-minute timeout per test.
3. Create `tests/e2e/env.ts` that reads and validates all required env vars: `CTO_PM_URL`, `LINEAR_API_KEY`, `GITHUB_TOKEN`, `DISCORD_BRIDGE_URL`, `HERMES_URL`, `NOUS_API_KEY`. Throw descriptive errors if critical vars are missing.
4. Create `tests/e2e/helpers.ts` with: `generateRunId()` (UUID-based), `waitForPipelineStatus(runId, targetStatus, timeoutMs)` polling helper, and typed response interfaces for pipeline status, delegation status, and PR state endpoints.
5. Create `tests/e2e/constants.ts` with URL patterns (e.g., `GITHUB_PR_PATTERN = /https:\/\/github\.com\/5dlabs\/sigma-1\/pull\/\d+/`).
6. Verify the test runner can import and execute a trivial test in the E2E directory.

## Validation
Run `bun test tests/e2e/` with a trivial assertion — it passes. All env var reads are validated. `generateRunId()` returns a unique string. `waitForPipelineStatus` compiles and its type signatures are correct.