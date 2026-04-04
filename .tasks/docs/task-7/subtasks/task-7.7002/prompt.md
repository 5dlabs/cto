Implement subtask 7002: Test assignee presence and correct delegate mapping on Linear issues

## Objective
Using the fetched issue data and delegate mapping from setup, write test cases asserting: (1) at least 80% of issues (minimum 4 of 5+) have a non-null assignee, and (2) each assigned issue's assignee.id matches the expected Linear user ID for that task's agent hint from the reference mapping.

## Steps
1. Test case 'Assignee presence': iterate over all fetched issues. Count those with `assignee !== null`. Assert `assignedCount >= Math.max(4, Math.ceil(totalIssues * 0.8))`.
2. Test case 'Correct mapping': for each issue that has an assignee, look up the agent hint associated with that issue ID (from the pipeline results loaded in setup). Then look up the expected Linear user ID from the delegate mapping. Assert `issue.assignee.id === expectedUserId`. Collect mismatches and report all at once for debuggability.
3. If no delegate mapping entry exists for a given agent hint, skip that issue in the mapping test but do NOT count it as a failure (it will be covered by the unassigned handling subtask).

## Validation
Test 'Assignee presence' passes with >= 80% of issues having non-null assignee. Test 'Correct mapping' passes with 0 mismatches between assignee.id and expected user ID for all mapped issues.