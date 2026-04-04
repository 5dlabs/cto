Implement subtask 3003: Implement availability gating for missing NOUS_API_KEY

## Objective
Add a pre-check that skips the Hermes research call entirely when NOUS_API_KEY is not set or empty, logging a structured skip event and producing a skip-type memo.

## Steps
1. In `src/clients/hermes.ts` or a new `src/services/hermes-research.ts` orchestrator, add an availability check function: `isHermesAvailable(): boolean` that returns `false` if `process.env.NOUS_API_KEY` is undefined, null, or an empty/whitespace-only string.
2. Define a skip memo type: `{ source: 'skipped', reason: 'no_api_key', content: null }`.
3. When the availability check fails, log at `info` level with structured fields `{ stage: 'hermes_research', action: 'skipped', reason: 'no_api_key' }` using the project's existing logger.
4. Return the skip memo immediately without instantiating the HTTP client or circuit breaker.
5. This check should be the first thing evaluated in the research orchestrator function before any circuit breaker or HTTP logic runs.

## Validation
Unit tests: (1) Unset NOUS_API_KEY entirely; call the orchestrator; assert it returns the skip memo and no HTTP calls are made (mock fetch and assert zero invocations). (2) Set NOUS_API_KEY to empty string; assert same behavior. (3) Set NOUS_API_KEY to whitespace-only; assert same behavior. (4) Set NOUS_API_KEY to a valid value; assert the check passes and execution continues to the client call. (5) Assert the structured log entry is emitted with the expected fields.