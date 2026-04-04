Implement subtask 8007: Implement AC-4 test: PR creation in sigma-1

## Objective
Write E2E test verifying that the pipeline created a PR in the sigma-1 repository with the correct URL pattern, containing tasks/ directory files and pipeline-meta.json.

## Steps
Step-by-step:
1. In `tests/e2e/pipeline.test.ts`, create test: `test('AC-4: PR created in sigma-1', async () => { ... })`.
2. Using the run ID from AC-1 test, query pipeline state for the PR URL (e.g., `GET /api/pipeline/:runId/status` and extract `prUrl` field).
3. Assert: `prUrl` is non-null and non-empty.
4. Assert: `prUrl` matches regex `/https:\/\/github\.com\/5dlabs\/sigma-1\/pull\/\d+/`.
5. Using the GitHubTestAdapter, call `githubAdapter.verifyPR(prUrl)`.
6. Assert: PR state is 'open'.
7. Assert: PR files include >= 5 files under the `tasks/` directory path.
8. Assert: PR files include a `pipeline-meta.json` file.
9. Store the PR URL for cleanup phase.

## Validation
Test passes when: PR URL matches the sigma-1 pattern, PR state is 'open' (live or mock), PR contains >= 5 files in tasks/ directory, and pipeline-meta.json is present. Test fails if PR URL is null or malformed.