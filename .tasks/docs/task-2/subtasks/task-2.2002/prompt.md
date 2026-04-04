Implement subtask 2002: Implement and verify resolve_agent_delegates() function

## Objective
Locate or implement the resolve_agent_delegates() function that accepts an array of agent hint strings and returns a mapping of agent hints to Linear user IDs, with null for unresolvable agents.

## Steps
1. Locate the existing `resolve_agent_delegates()` function in the PM server codebase.
2. Verify it accepts an array of agent hint strings (e.g., `['bolt', 'nova', 'blaze']`) and returns a `Record<string, string | null>` mapping each hint to a Linear user ID or null.
3. If the function does not exist, implement it:
   a. Define the agent-to-Linear-user-ID mapping (source TBD per decision point — hardcoded map, ConfigMap, or Linear API query).
   b. Accept `string[]` of agent hints.
   c. Return `Record<string, string | null>` with resolved IDs or null for unknown agents.
4. Ensure the function handles edge cases: empty array input, duplicate agent hints, case-insensitive matching.
5. The function should be a pure batch operation — resolve all hints in a single call.
6. Export the function for import by the pipeline integration code.

## Validation
Unit test: `resolve_agent_delegates(['bolt', 'nova', 'blaze'])` returns an object with 3 keys, each mapping to a non-empty string (Linear user ID). Unit test: `resolve_agent_delegates([])` returns an empty object. Unit test: `resolve_agent_delegates(['unknown_agent'])` returns `{ unknown_agent: null }`.