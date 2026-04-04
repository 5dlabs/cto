## Verify Linear Issues Have Delegate Assignments (Tess - Test frameworks)

### Objective
Validate that Linear issues created by the pipeline have the correct assignee set via delegate_id resolution. Query the Linear API directly to confirm assigneeId is populated and corresponds to the expected agent-to-user mapping, not just labels or custom fields.

### Ownership
- Agent: tess
- Stack: Test frameworks
- Priority: high
- Status: pending
- Dependencies: 2, 6

### Implementation Details
1. Create test file `e2e/linear-delegate-assignments.test.ts`.
2. Prerequisite: retrieve the `linearSessionId` and list of created issue IDs from the pipeline run results (from Task 6's pipeline output or a dedicated API endpoint).
3. Test case 1 — Assignee presence: for each created Linear issue, query the Linear GraphQL API (`issue { assignee { id name } }`). Assert: at least 4 out of 5+ issues have a non-null `assignee`.
4. Test case 2 — Correct mapping: maintain a reference mapping of agent hints to expected Linear user IDs (sourced from the delegate-status endpoint or a fixture). For each assigned issue, assert the `assignee.id` matches the expected Linear user ID for that task's agent.
5. Test case 3 — Unassigned handling: if any issues lack an assignee, assert they correspond to tasks with unknown/unmapped agent hints, and that the PM server logged an error for each.
6. Test case 4 — No "agent:pending" labels: assert none of the created issues have a label matching `agent:pending` or similar placeholder patterns.
7. Test case 5 — Issue metadata: assert each issue has a title, description, and is in the correct Linear project/team.
8. Use the `LINEAR_API_TOKEN` from environment for direct API queries. Do not go through the PM server for verification — query Linear independently to avoid circular validation.

### Subtasks
- [ ] Set up test file, Linear GraphQL client, and retrieve pipeline issue IDs: Create `e2e/linear-delegate-assignments.test.ts` with a configured Linear GraphQL client using `LINEAR_API_TOKEN` from environment. In the test setup (beforeAll), retrieve the list of created issue IDs from the Task 6 pipeline run output. Also fetch the reference agent-to-Linear-user-ID mapping from the delegate-status endpoint or a test fixture, storing both for use across all test cases.
- [ ] Test assignee presence and correct delegate mapping on Linear issues: Using the fetched issue data and delegate mapping from setup, write test cases asserting: (1) at least 80% of issues (minimum 4 of 5+) have a non-null assignee, and (2) each assigned issue's assignee.id matches the expected Linear user ID for that task's agent hint from the reference mapping.
- [ ] Test unassigned issue handling, label validation, and issue metadata: Write test cases covering edge cases and metadata: (1) unassigned issues correspond to unmapped agent hints and have matching PM server error logs, (2) no issues carry 'agent:pending' or similar placeholder labels, (3) all issues have non-empty titles, descriptions, and belong to the expected Linear project/team.