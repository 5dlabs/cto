Implement subtask 2008: Implement health check and Prometheus metrics endpoints

## Objective
Add /healthz, /readyz, and /metrics endpoints for Kubernetes probes and Prometheus scraping.

## Steps
1. Create a handlers/observability.rs module.
2. Implement GET /healthz: return 200 OK if the server is running (liveness probe).
3. Implement GET /readyz: check PostgreSQL and Redis connectivity, return 200 if both are reachable, 503 otherwise (readiness probe).
4. Add prometheus-client or metrics crate as dependency.
5. Instrument key metrics: http_requests_total (counter by method, path, status), http_request_duration_seconds (histogram), active_connections (gauge).
6. Implement GET /metrics: return Prometheus text format exposition.
7. Add Axum middleware to record request metrics on every request.
8. Update the Kubernetes Deployment manifest with liveness/readiness probe configurations pointing to /healthz and /readyz.

## Validation
GET /healthz returns 200. GET /readyz returns 200 when DB and Redis are up, 503 when either is down. GET /metrics returns valid Prometheus text format with http_requests_total and http_request_duration_seconds metrics. After making several API calls, metrics counters increment correctly.