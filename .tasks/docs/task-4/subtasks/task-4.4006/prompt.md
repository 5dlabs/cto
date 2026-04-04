Implement subtask 4006: Write integration test verifying full PR creation API call sequence

## Objective
Create an integration test that runs createSnapshotPR end-to-end with a fully mocked GitHub API and verifies the complete sequence of operations.

## Steps
1. Create `src/design-snapshot/__tests__/integration.test.ts`.
2. Set up a comprehensive GitHub API mock that records all incoming requests in order.
3. Construct a full PipelineOutput with multiple tasks (some with research memos, some without).
4. Call `createSnapshotPR(pipelineOutput)` with GITHUB_TOKEN set.
5. Assert the API call sequence: (a) GET ref to main, (b) POST ref to create pipeline branch, (c) POST blobs for each file, (d) POST tree, (e) POST commit, (f) PATCH ref to update branch, (g) POST pull request.
6. Verify the returned PRResult contains a valid prUrl.
7. Verify the task scaffold files have correct paths and content structure.
8. Verify the PR body formatting matches expected output.

## Validation
Integration test passes with all API calls made in the correct order. PRResult.prUrl is non-null. The number of blob creation calls matches the number of generated files. PR body contains correct task count and agent assignments.