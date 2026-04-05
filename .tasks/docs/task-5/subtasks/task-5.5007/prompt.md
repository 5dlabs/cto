Implement subtask 5007: Extract shared circuit breaker module

## Objective
Extract the circuit breaker pattern into a shared module `src/clients/circuit_breaker.rs` used by all four external API clients, avoiding code duplication.

## Steps
1. Create `src/clients/circuit_breaker.rs`.
2. Define `CircuitBreaker` struct:
   - state: `CircuitState` enum (Closed, Open, HalfOpen)
   - consecutive_failures: u32
   - failure_threshold: u32 (default 5)
   - cooldown: Duration (default 30s)
   - last_failure_at: Option<Instant>
3. Methods:
   - `pub fn new(failure_threshold: u32, cooldown: Duration) -> Self`
   - `pub fn is_available(&self) -> bool` — returns true if Closed, or if Open and past cooldown (transition to HalfOpen)
   - `pub fn record_success(&mut self)` — reset to Closed, zero failures
   - `pub fn record_failure(&mut self)` — increment failures, transition to Open if threshold reached
4. Wrap in `Arc<Mutex<CircuitBreaker>>` for async sharing.
5. Provide a helper: `pub async fn execute_with_circuit_breaker<F, T>(cb: &Arc<Mutex<CircuitBreaker>>, f: F) -> Result<T, VettingError>` that checks availability, runs the future, records success/failure.
6. Refactor all four clients (OpenCorporates, LinkedIn, GooglePlaces, Credit) to use this shared module.
7. Unit test the circuit breaker state machine independently: closed → 5 failures → open → wait cooldown → half-open → success → closed.

## Validation
Unit test: 5 consecutive record_failure calls transition state to Open. Unit test: in Open state, is_available returns false before cooldown. Unit test: after cooldown, is_available returns true (HalfOpen). Unit test: record_success in HalfOpen transitions to Closed. All four clients compile and pass their existing tests after refactor.