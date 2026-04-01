Implement subtask 8006: Implement Test Case 3: Linear Issue Delegation Verification with Cross-API Validation

## Objective
Write the E2E test that fetches Linear issues created by the pipeline, asserts correct count and delegate assignment, and cross-verifies each issue's assignee against the Linear API.

## Steps
1. Create `tests/e2e/helpers/linear-client.ts` — a minimal Linear API client using fetch:
   a. Constructor takes `LINEAR_API_KEY`.
   b. Method `getIssue(issueId: string)` calls Linear GraphQL API to fetch issue with `id`, `assignee { id, name }`.
2. Test Case 3 (`it('creates >= 5 Linear issues with correct delegate assignees')`):
   a. Call `GET ${PM_SERVER_URL}/api/pipeline/${runId}/issues`.
   b. Parse response as JSON array of issue objects.
   c. Assert: `issues.length >= 5`.
   d. Assert: every issue has a non-null `assigneeId` field.
   e. For each issue (or first 5 to limit API calls), call `linearClient.getIssue(issue.linearIssueId)`.
   f. Assert: the Linear API response's `assignee.id` matches the pipeline's `assigneeId`.
   g. Collect and log mismatches for debugging.
3. Handle Linear API rate limits: add a 200ms delay between API calls.
4. If LINEAR_API_KEY is not set, skip this test with a clear message.

## Validation
Test passes when >= 5 issues are returned, each has a non-null assigneeId, and cross-verification against Linear API confirms assignee matches for all checked issues. Test skips gracefully without LINEAR_API_KEY.