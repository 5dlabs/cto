<identity>
You are rex working on subtask 5005 of task 5.
</identity>

<context>
<scope>
Add an HTTP /healthz endpoint on port 8080 and configure structured JSON logging with tracing-subscriber, including metric counters for key events.
</scope>
</context>

<implementation_plan>
1. Add `hyper` (or `axum` if already available, or minimal `tokio` TCP listener) dependency for the health endpoint.
2. Implement `src/health.rs` with a minimal HTTP server on `:8080` serving a single route `/healthz`.
3. Health check logic: maintain a shared `Arc<AtomicBool>` for Redis connectivity and another for Solana RPC reachability. The `/healthz` endpoint returns 200 with `{"status":"ok"}` if both are true, 503 with `{"status":"degraded","redis":bool,"solana":bool}` otherwise.
4. Expose a `HealthState` struct that the consumer and submitter update: set redis_connected=true after successful XREADGROUP, set solana_reachable=true after successful RPC call, set false on failures.
5. Configure tracing-subscriber in main.rs: JSON format, with fields for timestamp, level, target, span. Include structured fields on key events: `events_processed` (counter incremented on successful XACK), `events_failed` (counter on DLQ write), `solana_rpc_errors` (counter on RPC failures), `retry_count` (per-event retry attempts).
6. Spawn the health server as a separate tokio task in main.rs alongside the consumer loop.
7. Ensure the health server shuts down gracefully with the rest of the application on SIGTERM.
</implementation_plan>

<validation>
Unit test: start health server, send HTTP GET to /healthz — verify 200 response with ok status when both flags are true. Set redis_connected=false, verify 503 response with degraded status. Integration: verify tracing output is valid JSON with expected fields by capturing log output in test and parsing it.
</validation>