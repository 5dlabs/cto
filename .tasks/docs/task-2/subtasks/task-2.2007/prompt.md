Implement subtask 2007: Implement Prometheus metrics and health probe endpoints

## Objective
Add /metrics endpoint exposing Prometheus-compatible metrics and /health/live, /health/ready endpoints for Kubernetes probes.

## Steps
1. Create src/handlers/health.rs:
   - GET /health/live: Return 200 {status: "ok"} unconditionally (liveness probe).
   - GET /health/ready: Check PostgreSQL connectivity (SELECT 1) and Redis connectivity (PING). Return 200 if both pass, 503 with details if either fails.
2. Create src/handlers/metrics.rs:
   - GET /metrics: Return Prometheus text format metrics.
   - Track: http_requests_total (counter, labels: method, path, status), http_request_duration_seconds (histogram, labels: method, path), active_connections (gauge), db_query_duration_seconds (histogram), rate_limit_hits_total (counter, labels: tenant_id).
3. Use prometheus-client or metrics + metrics-exporter-prometheus crate.
4. Create an Axum middleware layer that records request count and duration for every request.
5. Wire /metrics, /health/live, /health/ready routes (outside the rate-limited route group).
6. Update the Kubernetes Deployment manifest with liveness and readiness probe configurations pointing to these endpoints.

## Validation
GET /health/live returns 200. GET /health/ready returns 200 when DB and Redis are up, 503 when either is down. GET /metrics returns valid Prometheus text format. After sending several API requests, metrics show incremented counters and histogram observations. Kubernetes probes configured in the Deployment manifest point to correct paths and ports.