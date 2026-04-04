Implement subtask 3006: Add circuit breaker state to GET /api/pipeline/status health check

## Objective
Extend the existing pipeline status endpoint to include the Hermes circuit breaker state (closed/open/half-open) in its response payload.

## Steps
1. Locate the existing `GET /api/pipeline/status` route handler (likely in `src/routes/pipeline.ts` or similar Elysia route file).
2. Import the circuit breaker instance used by the Hermes research orchestrator. Expose it as a singleton or via a service registry so the route handler can call `getState()`.
3. Add a `hermes` field to the status response: `{ hermes: { circuitBreaker: 'closed' | 'open' | 'half-open', apiKeyConfigured: boolean } }`.
4. The `apiKeyConfigured` field reflects whether `NOUS_API_KEY` is set (boolean, not the key value — never expose secrets).
5. If the circuit breaker has not been initialized (e.g., API key not configured so it was never created), return `{ hermes: { circuitBreaker: 'n/a', apiKeyConfigured: false } }`.
6. Ensure the added fields do not break existing consumers of the status endpoint by only adding new keys, not modifying existing ones.

## Validation
Unit/integration tests: (1) Call GET /api/pipeline/status with circuit breaker in closed state; assert response includes `hermes.circuitBreaker: 'closed'`. (2) Trigger circuit breaker to open state, call status endpoint; assert `hermes.circuitBreaker: 'open'`. (3) With no NOUS_API_KEY set, call status endpoint; assert `hermes.circuitBreaker: 'n/a'` and `hermes.apiKeyConfigured: false`. (4) Assert existing fields in the status response are unchanged (backward compatibility). (5) Assert the API key value is never included in the response.