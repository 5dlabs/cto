Implement subtask 4007: Add Prometheus metrics and health endpoints

## Objective
Instrument the Finance service with Prometheus metrics for request counts, latencies, error rates, and business metrics, plus health and readiness probes.

## Steps
1. Add dependencies: metrics, metrics-exporter-prometheus (or axum-prometheus).
2. Create `src/middleware/metrics.rs`: Axum middleware layer that records for each request: finance_http_requests_total (method, path, status), finance_http_request_duration_seconds (method, path).
3. Add business metrics counters: finance_invoices_created_total, finance_payments_processed_total, finance_payments_failed_total, finance_stripe_webhook_events_total (event_type), finance_currency_sync_runs_total (status: success/failure), finance_reminders_sent_total.
4. Instrument each service module to increment relevant counters.
5. Create `src/routes/health.rs`:
   - GET /healthz — liveness probe: return 200 if process running
   - GET /readyz — readiness probe: check PostgreSQL pool (sqlx::query("SELECT 1")), check Redis (PING), return 200 only if both succeed, 503 otherwise
   - GET /metrics — Prometheus-formatted metrics export
6. Register metrics middleware in Axum router.
7. Add tracing spans to key service methods for observability.

## Validation
Verify /healthz returns 200. Verify /readyz returns 200 with healthy dependencies, 503 when PostgreSQL or Redis is unreachable. Make API calls across all endpoints, then verify /metrics contains expected counters with correct labels and incremented values. Verify histogram buckets are populated for request duration.