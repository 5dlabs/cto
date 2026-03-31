Implement subtask 2010: Implement Prometheus metrics and health probe endpoints

## Objective
Add /metrics endpoint exporting Prometheus-format metrics and /healthz, /readyz endpoints for Kubernetes liveness and readiness probes.

## Steps
1. Add `metrics`, `metrics-exporter-prometheus` or `axum-prometheus` crate to Cargo.toml. 2. Create src/handlers/health_handlers.rs. 3. GET /healthz (liveness): Return 200 {"status": "ok"} if the process is running. Lightweight, no dependency checks. 4. GET /readyz (readiness): Check PostgreSQL connectivity (SELECT 1) and Redis connectivity (PING). Return 200 if both succeed, 503 if either fails, with details: {"status": "ready|degraded", "checks": {"postgres": "ok|error", "redis": "ok|error"}}. 5. GET /metrics: Expose Prometheus metrics including: http_requests_total (counter, labels: method, path, status), http_request_duration_seconds (histogram, labels: method, path), db_pool_connections_active (gauge), db_query_duration_seconds (histogram). 6. Add a middleware layer that records request count and duration for every request. 7. Register health and metrics routes outside the /api/v1 prefix (at the root level).

## Validation
GET /healthz returns 200. GET /readyz returns 200 when both PostgreSQL and Redis are available, 503 when either is down. GET /metrics returns text/plain in Prometheus exposition format with http_requests_total and http_request_duration_seconds metrics present. After making catalog API calls, metric counters increment correctly.