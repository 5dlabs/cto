Implement subtask 2007: Add Prometheus metrics, health probes, and performance validation

## Objective
Implement Prometheus metrics exposition, Kubernetes health probes (liveness and readiness), and validate that availability checks complete in <500ms.

## Steps
1. Add `metrics` and `metrics-exporter-prometheus` crates (or `axum-prometheus`).
2. Implement `GET /metrics` endpoint exposing Prometheus-format metrics:
   - http_requests_total (counter, labeled by method, path, status)
   - http_request_duration_seconds (histogram, labeled by method, path)
   - db_pool_connections_active (gauge)
   - redis_connections_active (gauge)
3. Implement `GET /health/live`:
   - Return 200 if the process is running (simple liveness check).
4. Implement `GET /health/ready`:
   - Check PostgreSQL connectivity (SELECT 1).
   - Check Redis connectivity (PING).
   - Return 200 only if all dependencies are reachable, 503 otherwise.
5. Add middleware to record request duration and count for all routes.
6. Performance validation:
   - Add a database index on availability(product_id, date) if not already present.
   - Profile the availability endpoint query and ensure it completes in <500ms.
   - Add a request timeout of 5 seconds to the Axum server.
7. Create a Dockerfile for the service (multi-stage build: builder with rust:1.75 → runtime with debian-slim).
8. Create a Kubernetes Deployment manifest referencing sigma1-infra-endpoints ConfigMap via envFrom.

## Validation
Verify /metrics returns valid Prometheus format with expected metric names. Verify /health/live returns 200. Verify /health/ready returns 200 when dependencies are up and 503 when PostgreSQL or Redis is down. Load test the availability endpoint with 100 concurrent requests and verify p99 latency is <500ms. Verify the Docker image builds and runs successfully.