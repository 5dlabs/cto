Implement subtask 3004: Implement fallback behavior for circuit-open, timeout, and error scenarios

## Objective
Create the fallback memo generator that produces a structured default research memo when the Hermes call fails for any reason, with differentiated reason codes and warn-level structured logging.

## Steps
1. Create `src/services/hermes-fallback.ts` exporting a `createFallbackMemo(reason: 'hermes_unavailable' | 'circuit_open' | 'timeout'): ResearchMemo` function.
2. The fallback memo shape: `{ source: 'fallback', reason: string, content: null }`.
3. Map error types to reasons: circuit breaker open rejection → 'circuit_open', timeout/abort error → 'timeout', any other HTTP or parse error → 'hermes_unavailable'.
4. Log at `warn` level with structured fields: `{ stage: 'hermes_research', action: 'fallback', reason: <reason>, error_message: <original error message> }`.
5. Wire the fallback into the circuit breaker's rejection path: when the wrapped Hermes call rejects (for any reason), catch the error, classify it, and return the fallback memo instead of propagating the error.
6. Ensure the fallback path is synchronous and non-blocking — it must not introduce any additional async operations.

## Validation
Unit tests: (1) Simulate a circuit-open rejection; assert fallback memo has `reason: 'circuit_open'` and warn log is emitted. (2) Simulate a timeout error (AbortError); assert fallback memo has `reason: 'timeout'`. (3) Simulate a generic HTTP 500 error; assert fallback memo has `reason: 'hermes_unavailable'`. (4) Assert all fallback memos have `source: 'fallback'` and `content: null`. (5) Assert the fallback function completes synchronously (no await, no pending promises).