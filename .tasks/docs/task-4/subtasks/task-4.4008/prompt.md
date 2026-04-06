Implement subtask 4008: Implement Prometheus metrics and health/readiness endpoints

## Objective
Add Prometheus metrics instrumentation to all endpoints and implement /healthz and /readyz endpoints for the Finance service.

## Steps
1. Add prometheus and metrics crate dependencies. 2. Create `src/metrics.rs`: define metrics — http_requests_total (counter, labels: method, path, status), http_request_duration_seconds (histogram, labels: method, path), stripe_api_calls_total (counter, labels: operation, status), currency_sync_runs_total (counter, labels: status), active_db_connections (gauge). 3. Implement Axum middleware layer that records request count and duration for every request. 4. Add metrics increment calls in Stripe client and currency sync job. 5. Implement GET /metrics endpoint exposing Prometheus text format. 6. Implement GET /healthz: check PostgreSQL connection (SELECT 1), check Redis PING, return JSON {status: 'healthy'} or {status: 'unhealthy', details: [...]}. 7. Implement GET /readyz: same as healthz plus verify migrations are applied (query sqlx migration table), return 200 or 503. 8. Wire all observability routes into main router outside of /api/v1 prefix.

## Validation
/metrics returns valid Prometheus text format with all defined metrics; /healthz returns 200 when PostgreSQL and Redis are up, returns 503 with details when a dependency is down; /readyz returns 503 before migrations and 200 after; http_requests_total increments correctly after API calls; histogram buckets capture request latencies.