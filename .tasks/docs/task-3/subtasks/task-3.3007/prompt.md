Implement subtask 3007: Write comprehensive test suite for the Hermes integration

## Objective
Create end-to-end and edge-case tests covering the full Hermes research integration: happy path, timeout, circuit breaker state transitions, no-API-key gating, and artifact persistence.

## Steps
1. Create `tests/hermes-integration.test.ts` with a test suite that exercises the full deliberation pipeline with various Hermes scenarios.
2. Happy path: stub Hermes to return valid research; run deliberation; assert `researchMemo.source === 'hermes'` and content is non-empty and persisted.
3. Timeout test: stub Hermes with a 35s delay; assert the circuit breaker times out at ~30s, fallback memo is generated, and total wall time is under 31s.
4. Circuit breaker full cycle: trigger 3 consecutive failures; assert 4th call gets immediate fallback (no HTTP); wait 60s (or mock time); assert half-open probe is attempted; on probe success, assert state returns to closed.
5. No-API-key test: unset NOUS_API_KEY; run deliberation; assert skip memo in artifacts and zero HTTP calls to Hermes endpoint.
6. Concurrent request test: fire 5 deliberation requests simultaneously with Hermes stubbed to succeed; assert all 5 get valid research memos and the circuit breaker state remains closed.
7. Health check reflects state: after triggering open circuit, assert GET /api/pipeline/status returns `hermes.circuitBreaker: 'open'`.

## Validation
This IS the test subtask. Verify by running the full test suite and confirming all tests pass with 100% of the defined scenarios covered. Check test coverage report to ensure hermes.ts, circuit-breaker.ts, hermes-fallback.ts, and the deliberation pipeline integration points are covered at >90% line coverage.