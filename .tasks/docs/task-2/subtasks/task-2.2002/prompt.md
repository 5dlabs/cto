Implement subtask 2002: Implement resolve_agent_delegates() function with agent-to-Linear-user-ID mapping

## Objective
Create the resolve_agent_delegates() function that accepts an array of agent hint strings, looks up each against an agent-to-Linear-user-ID mapping, and returns a Map<string, string | undefined>. Unknown or invalid agents must return undefined.

## Steps
1. Create `src/delegates/resolve-agent-delegates.ts`.
2. Define the mapping source: load a `DELEGATE_MAP` from environment variables or a JSON config (e.g., `DELEGATE_MAP_NOVA=lin_user_xxx`). Parse at module load time into a `Record<string, string>`.
3. Export `resolve_agent_delegates(agentHints: string[]): Map<string, string | undefined>` — iterate hints, look up each in the parsed map, return the Map.
4. For any hint not found in the map, the Map entry should be `undefined`.
5. Log at `debug` level: `{ stage: 'delegate_resolution', mapped: number, unmapped: string[] }` summarizing the resolution result.
6. Export the raw mapping as `getDelegateMap(): Record<string, string>` for use by the observability endpoint.

## Validation
Unit test: call resolve_agent_delegates(['nova', 'bolt', 'unknown_agent']) with a known test mapping. Assert 'nova' and 'bolt' resolve to correct Linear user IDs and 'unknown_agent' resolves to undefined. Assert getDelegateMap() returns the full mapping object.