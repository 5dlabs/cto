Implement subtask 5010: Add Prometheus metrics, health checks, and OpenAPI documentation

## Objective
Integrate Prometheus metrics collection, liveness/readiness health check endpoints, and generate OpenAPI specification for the vetting service.

## Steps
1. Add dependencies: `axum-prometheus` or `metrics`/`metrics-exporter-prometheus`, `utoipa` for OpenAPI.
2. Implement `GET /healthz` (liveness) that returns HTTP 200 if the server is running.
3. Implement `GET /readyz` (readiness) that checks database connectivity and returns HTTP 200 or 503.
4. Add Prometheus metrics:
   - `vetting_requests_total` (counter, labels: endpoint, status)
   - `vetting_pipeline_duration_seconds` (histogram)
   - `vetting_stage_duration_seconds` (histogram, label: stage_name)
   - `vetting_score_distribution` (counter, labels: grade)
   - Expose at `GET /metrics`.
5. Annotate all endpoint handlers with `utoipa` macros to generate OpenAPI 3.0 spec.
6. Serve the OpenAPI JSON at `GET /api/v1/vetting/openapi.json`.
7. Integrate metrics middleware into the Axum router to automatically track request counts and latencies.

## Validation
GET /healthz returns 200. GET /readyz returns 200 when DB is up, 503 when DB is down. GET /metrics returns Prometheus-formatted output containing the defined metric names. GET /api/v1/vetting/openapi.json returns valid OpenAPI 3.0 JSON with all three endpoints documented.