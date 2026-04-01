Implement subtask 2001: Implement resolve_agent_delegates() function with Linear Users API query

## Objective
Create a new TypeScript module exporting resolve_agent_delegates(agentHints: string[]) that queries the Linear GraphQL users endpoint, matches agent hint strings to Linear user profiles, and returns a Map<string, string> of agentHint → linearUserId.

## Steps
1. Create a new file `src/lib/resolve-agent-delegates.ts`.
2. Define the function signature: `async function resolve_agent_delegates(agentHints: string[]): Promise<Map<string, string>>`.
3. Use the Linear GraphQL API (`users` query) to fetch all workspace users. Filter/match each agent hint to a user by display name (or chosen matching strategy per decision point).
4. For any hint that cannot be resolved, log a structured warning (agent hint, reason) and exclude it from the returned map — do NOT throw.
5. Return the populated map.
6. Export the function as a named export for consumption by the issue creation module.
7. Ensure the Linear API token is read from environment variables sourced via `sigma1-infra-endpoints` ConfigMap (`envFrom`).

## Validation
Unit test with a mocked Linear GraphQL client returning 3 known users. Verify: (a) all 3 known hints resolve correctly, (b) an unknown hint returns no entry and produces a warning log, (c) function returns a Map with expected size.