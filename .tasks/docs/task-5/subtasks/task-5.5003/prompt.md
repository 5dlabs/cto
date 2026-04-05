Implement subtask 5003: Build HTTP client utilities: timeout, retry with backoff, and circuit breaker

## Objective
Implement a reusable HTTP client wrapper providing per-request 10-second timeouts, 2 retries with exponential backoff, and a circuit breaker that opens after 5 failures in 5 minutes, marking the stage as unavailable instead of failing the pipeline.

## Steps
1. Create `src/http_client.rs` module with a `ResilientClient` struct wrapping `reqwest::Client`.
2. Implement `execute_with_retry` method: accepts a request builder closure, retries up to 2 times on transient errors (5xx, timeout, connection refused) with exponential backoff (1s, 2s).
3. Set per-request timeout to 10 seconds using `reqwest::ClientBuilder::timeout` or per-request `.timeout(Duration::from_secs(10))`.
4. Implement `CircuitBreaker` struct: tracks failure count and timestamps in an `Arc<Mutex<CircuitBreakerState>>`. State includes failure_count, last_failure_at, is_open. If 5 failures within 5 minutes, set is_open=true. Half-open check: after 1 minute, allow one request through.
5. Create a `CircuitBreakerRegistry` keyed by stage name (String) so each external API has its own circuit breaker.
6. Define `StageResult` enum: `Success(serde_json::Value)`, `Unavailable(String)` — so callers can distinguish between a real result and a gracefully degraded one.
7. When circuit is open, immediately return `StageResult::Unavailable` without making the HTTP call.
8. All responses (success or error) should be capturable as raw JSON for the audit trail.

## Validation
Unit test: verify retry logic fires exactly 2 retries on 500 responses. Verify timeout triggers after 10s with a mock delayed response (use tokio::time::pause). Circuit breaker test: send 5 failing requests, verify 6th returns Unavailable without HTTP call. Verify half-open behavior after simulated 1-minute wait. Verify successful call resets failure count.