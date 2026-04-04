Implement subtask 2001: Audit and extend resolve_agent_delegates() mapping to cover 5+ agents

## Objective
Audit the existing resolve_agent_delegates() function in cto/cto-pm to determine the current mapping source and coverage. Extend it to map at least 5 distinct agent hints (bolt, nova, blaze, tess, and at least one more) to their corresponding Linear user IDs.

## Steps
1. Locate the resolve_agent_delegates() function in the cto-pm codebase.
2. Identify how agent → Linear user ID mappings are currently stored (hardcoded map, config, or API).
3. Document the current set of mappings and identify gaps.
4. Add missing agent entries so at least 5 agents are covered: bolt, nova, blaze, tess, and one additional (e.g., rex, grizz, or cipher).
5. Ensure the function signature accepts an agent hint string and returns `string | null` (Linear user ID or null for unknown agents).
6. Ensure the function reads Linear API token from the environment (injected via sigma-1-infra-endpoints ConfigMap envFrom from ExternalSecret sigma-1-linear-token).
7. Add structured logging: log the agent hint received and the resolved user ID or null.

## Validation
Unit test: call resolve_agent_delegates() with each of the 5 known agent hints and assert a non-null, valid-format Linear user ID is returned for each. Call with 'unknown-agent' and assert null is returned. Verify structured log output includes agent hint and resolution result.