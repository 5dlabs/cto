Implement subtask 7007: Write comprehensive unit and integration tests for the PR generation pipeline

## Objective
Create a dedicated test suite covering all GitHub API interaction sequences, scaffold content validation, error scenarios, and an end-to-end integration test verifying PR creation against the real 5dlabs/sigma-1 repo.

## Steps
1. Create test files in `src/services/__tests__/` for each module: `github-client.test.ts`, `branch-creator.test.ts`, `scaffold-generator.test.ts`, `git-committer.test.ts`, `pr-creator.test.ts`, `pr-generator.test.ts`.
2. Use Bun's built-in test runner (`bun:test`).
3. Mock `fetch` globally for unit tests using Bun's mock utilities.
4. Key test scenarios (in addition to per-subtask tests):
   a. Full sequence test: mock all GitHub API endpoints; run PRGenerator.generatePR; verify the exact sequence of API calls (GET ref → POST refs → POST blobs × N → POST trees → POST commits → PATCH refs → POST pulls).
   b. Content validation: generate scaffolds for 5 diverse tasks (different agents, stacks); verify each README contains all required sections.
   c. SUMMARY.md: verify table row count matches task count, dependency graph is complete.
   d. Error resilience: simulate network timeout on blob creation; verify clean error return.
   e. Edge case: 0 tasks — verify behavior (either skip PR or create empty scaffold).
5. Integration test (gated behind `INTEGRATION_TEST=true` env var): run against real 5dlabs/sigma-1 with a test runId; verify PR is created, then clean up (close PR, delete branch).

## Validation
Meta: Run `bun test` and verify all test files pass. Verify test coverage for each service module exceeds 90% line coverage. Verify integration test creates and cleans up a real PR when INTEGRATION_TEST=true is set.