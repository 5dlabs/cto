Implement subtask 2006: Add Prometheus metrics endpoint and Kubernetes health probes

## Objective
Implement /metrics endpoint exposing Prometheus-format metrics, and /health/live and /health/ready endpoints for Kubernetes liveness and readiness probes.

## Steps
1. Add dependencies: prometheus (or metrics + metrics-exporter-prometheus) crate.
2. Create src/handlers/observability.rs.
3. GET /metrics: expose Prometheus text format metrics including:
   - http_requests_total (counter, labels: method, path, status)
   - http_request_duration_seconds (histogram, labels: method, path)
   - db_pool_connections_active (gauge)
   - db_pool_connections_idle (gauge)
4. Create a Tower middleware that records request count and duration for every request, updating the Prometheus registry.
5. GET /health/live → liveness: return 200 { status: 'ok' } always (proves the process is running and not deadlocked).
6. GET /health/ready → readiness: check PostgreSQL connectivity (SELECT 1) and Redis connectivity (PING). Return 200 { status: 'ready', checks: { postgres: 'ok', redis: 'ok' } } if both pass, 503 with details if either fails.
7. Register /metrics, /health/live, /health/ready routes outside the /api/v1 prefix (at root level).
8. Ensure /metrics and /health/* endpoints are NOT rate limited.

## Validation
GET /metrics returns valid Prometheus text format with all four metric families; GET /health/live returns 200; GET /health/ready returns 200 when infra is healthy and 503 when PostgreSQL or Redis is unreachable; after sending API requests, /metrics shows incremented counters and non-zero histograms.