Implement subtask 6003: Implement Linear API verification for issue assignees

## Objective
After Linear issues are created in Stage 5, query the Linear API for each created issue to independently confirm that the `assignee.id` matches the expected `delegate_id` from the delegation resolution stage.

## Steps
1. Create `src/validation/linear-verifier.ts`.
2. Define `LinearVerificationResult` type: `{ issueId: string, title: string, expectedDelegateId: string, actualAssigneeId: string | null, matched: boolean, linearUrl: string }`.
3. Implement `verifyLinearAssignees(issues: CreatedIssue[]): Promise<LinearVerificationResult[]>`:
   - For each created issue, call Linear API `GET /issues/{id}` (or use the Linear SDK `linearClient.issue(id)`).
   - Extract `assignee.id` from the response.
   - Compare against the `delegate_id` that was set during creation.
   - Build the result object with `matched: actualAssigneeId === expectedDelegateId`.
   - Apply 100ms delay between API calls to respect rate limits.
4. Implement `summarizeVerification(results: LinearVerificationResult[]): { total: number, matched: number, unmatched: number, pendingOnly: number }`.
5. Detect issues that have ONLY the `agent:pending` label but whose agent hint has a known mapping — flag these as errors, not just warnings.
6. Export both functions for use by the report generator.

## Validation
Mock the Linear SDK client. Verify that for 5 mock issues with known assignees, `verifyLinearAssignees` returns 5 results with `matched: true`. Verify that an issue with a null assignee returns `matched: false`. Verify that `summarizeVerification` produces correct counts. Verify rate-limit delay is applied between calls.