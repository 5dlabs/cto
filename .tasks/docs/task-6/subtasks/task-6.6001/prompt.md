Implement subtask 6001: Detect existing test framework and configure E2E test scaffold

## Objective
Inspect the repository for the configured test framework (Jest, Vitest, or other), identify the existing test directory convention, and create the foundational test file `e2e/pipeline-completion.test.ts` with proper imports, describe blocks, global timeout configuration (360s), and environment variable documentation.

## Steps
1. Check `package.json` for test runner dependencies (jest, vitest, @jest/globals, etc.) and scripts.
2. Check for existing config files: `jest.config.*`, `vitest.config.*`, or test configuration in `package.json`.
3. Identify existing test directory structure (e.g., `__tests__/`, `tests/`, `e2e/`, `test/`).
4. Create `e2e/pipeline-completion.test.ts` (or match existing convention) with:
   - Header comment block documenting required env vars: `PM_SERVER_URL`, `PM_AUTH_TOKEN` (or equivalent), `TEST_PRD_PATH` (optional).
   - Framework-appropriate imports.
   - Top-level `describe('Pipeline E2E Completion')` block.
   - Global/suite-level timeout set to 360000ms.
   - Helper function `getPmServerUrl()` that reads `PM_SERVER_URL` env var and throws if missing.
   - Helper function `getAuthHeaders()` that constructs auth headers from env vars.
5. Create a sample PRD fixture file at `e2e/fixtures/sample-prd.json` with a minimal but valid PRD payload for testing.
6. If a JUnit reporter is not already configured, add the appropriate reporter package (e.g., `jest-junit` or `vitest-junit-reporter`) as a dev dependency and configure it to output to `test-results/e2e-pipeline.xml`.

## Validation
Running the test command (e.g., `npm test -- e2e/pipeline-completion`) succeeds with 0 test cases (empty describe block). The test file compiles without TypeScript errors. JUnit reporter config is present and would produce XML output. Required env var documentation is present in the file header comment.