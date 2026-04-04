Implement subtask 3002: Implement circuit breaker wrapper with configurable timeout, failure threshold, and reset

## Objective
Create a reusable circuit breaker module that wraps async functions with open/closed/half-open state management, configurable via environment variables. Evaluate opossum vs. cockatiel vs. a lightweight custom implementation for Bun compatibility.

## Steps
1. Research Bun compatibility: check if `opossum` works under Bun by reviewing its dependency on Node EventEmitter. If incompatible, evaluate `cockatiel` (Promise-based, no Node-specific APIs) or implement a minimal circuit breaker (~100 LOC).
2. Create `src/lib/circuit-breaker.ts` that exports a `createCircuitBreaker<T>(fn: () => Promise<T>, options: CircuitBreakerOptions): WrappedFn<T>` factory.
3. Define `CircuitBreakerOptions { timeout: number; failureThreshold: number; resetTimeout: number }` with defaults from env: `HERMES_TIMEOUT_MS` (default 30000), `HERMES_CB_FAILURE_THRESHOLD` (default 3), `HERMES_CB_RESET_MS` (default 60000).
4. Implement state machine: CLOSED (normal operation, count consecutive failures) → OPEN (reject immediately, start reset timer) → HALF_OPEN (allow one probe call, success → CLOSED, failure → OPEN).
5. Expose a `getState(): 'closed' | 'open' | 'half-open'` method for the health check integration.
6. Ensure the timeout is implemented via `AbortController` + `setTimeout` to properly abort the underlying fetch if it exceeds the configured timeout.
7. If using a third-party library, add it to `package.json` and document the rationale in a code comment.

## Validation
Unit tests: (1) Wrap a function that resolves; assert it returns the value and state remains 'closed'. (2) Wrap a function that rejects 3 times consecutively; assert state transitions to 'open' after the 3rd failure. (3) In 'open' state, call the wrapped function; assert it rejects immediately without invoking the inner function. (4) After resetTimeout elapses, assert state is 'half-open' and one probe call is allowed. (5) In 'half-open', a successful probe transitions to 'closed'. (6) In 'half-open', a failed probe transitions back to 'open'. (7) Wrap a function with a 100ms timeout around a 200ms delay; assert it rejects with a timeout error.