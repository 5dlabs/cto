Implement subtask 4007: Add Prometheus metrics, health endpoints, and OpenAPI documentation

## Objective
Implement observability with Prometheus metrics, health/readiness probes, and generate OpenAPI documentation for all finance endpoints.

## Steps
1. Add metrics dependencies: metrics, metrics-exporter-prometheus, axum-prometheus or custom tower layer. 2. Implement a metrics middleware layer that records: request_count (by method, path, status), request_duration_seconds histogram, active_connections gauge. Add custom finance metrics: invoices_created_total, payments_processed_total, payment_failures_total. 3. Expose GET /metrics endpoint with Prometheus text format. 4. Implement GET /healthz (liveness): returns 200 if service is running. 5. Implement GET /readyz (readiness): checks PostgreSQL connection pool and Redis connectivity, returns 200 if both healthy, 503 otherwise. 6. Generate OpenAPI spec: use utoipa crate to annotate all route handlers with OpenAPI metadata. Generate and serve /api/v1/finance/openapi.json. Optionally serve Swagger UI at /api/v1/finance/docs. 7. Verify all endpoints are documented with request/response schemas, status codes, and descriptions.

## Validation
GET /metrics returns valid Prometheus format with expected metric names; /healthz returns 200; /readyz returns 503 when PostgreSQL is down; OpenAPI JSON is valid and documents all endpoints with correct schemas; Swagger UI (if served) renders without errors.