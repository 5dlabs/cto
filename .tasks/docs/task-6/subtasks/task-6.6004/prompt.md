Implement subtask 6004: Implement Linear session and PR creation validation tests

## Objective
Implement test case 4 (Linear session — linearSessionId non-null, issueCount >= 5) and test case 5 (PR creation — prUrl matches GitHub pattern).

## Steps
1. Test case 4 — `it('should create a Linear session with at least 5 issues')`:
   - Query pipeline run results for Linear session metadata. Try `GET ${PM_SERVER_URL}/api/pipeline/runs/${pipelineRunId}` and inspect the response body for `linearSessionId` and `issueCount` fields, or check a dedicated endpoint like `/api/pipeline/runs/${pipelineRunId}/linear`.
   - Assert `linearSessionId` is a non-null, non-empty string.
   - Assert `issueCount >= 5`.
   - Include actual `issueCount` in assertion message.
2. Test case 5 — `it('should create a PR in the sigma-1 repository')`:
   - From the same pipeline run result (or a dedicated endpoint like `/api/pipeline/runs/${pipelineRunId}/pr`), extract `prUrl`.
   - Assert `prUrl` is a non-null, non-empty string.
   - Assert `prUrl` matches the regex `/^https:\/\/github\.com\/5dlabs\/sigma-1\/pull\/\d+$/`.
   - Include actual `prUrl` in assertion message on failure.
3. Both tests depend only on the pipeline having completed (from 6002's `beforeAll`), not on the task validation tests, so they can run in parallel with 6003.

## Validation
Test case 4 passes: `linearSessionId` is non-null and `issueCount >= 5`. Test case 5 passes: `prUrl` is non-null and matches `https://github.com/5dlabs/sigma-1/pull/\d+`. Both assertions include actual values in failure messages.