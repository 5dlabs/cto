Implement subtask 7003: Test unassigned issue handling, label validation, and issue metadata

## Objective
Write test cases covering edge cases and metadata: (1) unassigned issues correspond to unmapped agent hints and have matching PM server error logs, (2) no issues carry 'agent:pending' or similar placeholder labels, (3) all issues have non-empty titles, descriptions, and belong to the expected Linear project/team.

## Steps
1. Test case 'Unassigned handling': collect all issues where `assignee === null`. For each, assert the associated agent hint is NOT present in the delegate mapping (i.e., it's genuinely unmapped). Additionally, query PM server logs (via log file or logging endpoint) and assert that an error entry exists for each unassigned issue ID mentioning the unmapped agent hint. If no unassigned issues exist, the test should pass trivially.
2. Test case 'No agent:pending labels': for each issue, iterate `labels.nodes` and assert no label name matches patterns like `agent:pending`, `agent-pending`, `pending-assignment`, or similar placeholders. Use a regex like `/^agent[:\-_]?pending$/i` for flexibility.
3. Test case 'Issue metadata — title and description': for each issue, assert `issue.title` is a non-empty string with length > 0 and `issue.description` is a non-empty string with length > 0.
4. Test case 'Issue metadata — project/team': define the expected project ID and team ID (from env vars or fixture). For each issue, assert `issue.project.id === expectedProjectId` and `issue.team.id === expectedTeamId`.

## Validation
Unassigned issues (if any) all correspond to unmapped agent hints with matching error log entries. Zero issues have labels matching agent:pending patterns. All issues have non-empty title and description. All issues belong to the expected Linear project and team.