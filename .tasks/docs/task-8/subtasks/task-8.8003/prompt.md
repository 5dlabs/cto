Implement subtask 8003: Implement environment-aware test orchestration and path coverage guarantee

## Objective
Add orchestration logic that detects API key availability, conditionally routes to the correct test path, and includes a meta-assertion ensuring at least one of the two paths (primary or fallback) executed and passed in the current run.

## Steps
1. At the top of `e2e/hermes-research-validation.test.ts`, add a shared state tracker (e.g., `let primaryPathRan = false; let fallbackPathRan = false;`) that gets set to `true` inside the `beforeAll` of each conditional describe block.
2. Add a final top-level `describe('Path coverage guarantee')` block that runs unconditionally:
   a. **Test: At least one path validated** — `afterAll` or final test asserts `primaryPathRan || fallbackPathRan === true`. This ensures no silent skip of all tests.
   b. **Test: Environment detection is consistent** — Assert that `hasApiKey()` returns a stable boolean (call it twice, compare). Log which path was selected for CI debugging.
3. Add a descriptive console log at suite start: `'Running Hermes validation in mode: ${hasApiKey() ? "primary (with key)" : "fallback (no key)"}'`.
4. Review the conditional skip mechanism to ensure the test framework reports skipped tests as `skipped` (not `passed` or `failed`). For example, in Vitest use `describe.skipIf()`; in Jest use `describe.skip` conditionally.
5. Ensure the test file exports or documents the two environment configurations so CI can run the suite twice (once with key, once without) for full coverage if desired.

## Validation
Run the full test suite in both configurations (with and without NOUS_API_KEY). In each run: exactly one path's tests pass, the other path's tests are reported as skipped (not failed), and the path coverage guarantee test passes. Confirm the meta-assertion fails if both paths are somehow skipped (e.g., by temporarily breaking the hasApiKey helper).