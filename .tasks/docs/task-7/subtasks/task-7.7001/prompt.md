Implement subtask 7001: Set up test file, Linear GraphQL client, and retrieve pipeline issue IDs

## Objective
Create `e2e/linear-delegate-assignments.test.ts` with a configured Linear GraphQL client using `LINEAR_API_TOKEN` from environment. In the test setup (beforeAll), retrieve the list of created issue IDs from the Task 6 pipeline run output. Also fetch the reference agent-to-Linear-user-ID mapping from the delegate-status endpoint or a test fixture, storing both for use across all test cases.

## Steps
1. Create `e2e/linear-delegate-assignments.test.ts`.
2. Import or create a lightweight GraphQL client that hits `https://api.linear.app/graphql` with the `LINEAR_API_TOKEN` bearer token.
3. In `beforeAll`, load the pipeline run results (issue IDs and their associated agent hints) — either from a shared test artifact file written by Task 6, or by querying a known endpoint.
4. Also in `beforeAll`, load the delegate mapping (agent hint → expected Linear user ID). This can come from the delegate-status endpoint or a co-located fixture file `e2e/fixtures/delegate-mapping.json`.
5. For each issue ID, execute a GraphQL query: `query($id: String!) { issue(id: $id) { id title description assignee { id name } labels { nodes { name } } project { id name } team { id name } } }` and store the full response objects for assertion in sibling subtasks.
6. Validate the setup itself: assert that at least 5 issue IDs were retrieved and that the GraphQL responses are non-error.

## Validation
beforeAll completes without errors; at least 5 issue IDs are loaded; GraphQL responses for all issues return valid data (no error fields); delegate mapping contains at least 4 entries.