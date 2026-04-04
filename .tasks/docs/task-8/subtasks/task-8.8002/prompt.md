Implement subtask 8002: Implement mock/live adapters for Linear API

## Objective
Create an adapter layer for Linear API interactions that supports both live API calls and recorded mock responses. The adapter must support: creating test issues, querying issues by assignee/delegate_id, and cleanup (close/archive). Implement the mock variant with realistic recorded payloads.

## Steps
Step-by-step:
1. Create `tests/e2e/adapters/linear.ts` with interface `LinearTestAdapter` containing methods: `getIssuesForSession(sessionId: string)`, `verifyAssignee(issueId: string)`, `cleanup(issueIds: string[])`.
2. Implement `LiveLinearAdapter` class: uses `LINEAR_API_KEY` env var, queries Linear GraphQL API, implements cleanup by archiving issues.
3. Implement `MockLinearAdapter` class: returns recorded payloads from `tests/e2e/fixtures/linear/` directory. Create fixture files with realistic Linear issue responses (>= 5 issues, each with `assignee.id` set).
4. Create factory function `createLinearAdapter()` that selects live vs mock based on `E2E_LINEAR_MODE` env var (default: 'mock').
5. Ensure live adapter uses unique identifiers per test run to avoid cross-contamination.

## Validation
MockLinearAdapter returns fixture data with >= 5 issues each having non-null assignee.id. LiveLinearAdapter (if token available) can query the Linear API without errors. Factory function correctly selects adapter based on env var.