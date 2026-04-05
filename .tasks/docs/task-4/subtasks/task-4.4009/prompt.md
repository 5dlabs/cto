Implement subtask 4009: Add Prometheus metrics, health endpoints, and OpenAPI documentation

## Objective
Instrument the service with Prometheus metrics, implement readiness/liveness probes, and generate/document the OpenAPI specification for all endpoints.

## Steps
1. Add metrics dependencies: metrics, metrics-exporter-prometheus, axum-prometheus or custom tower middleware. 2. Add middleware layer that records request count, latency histogram, and response status for all routes. 3. Add custom business metrics: invoices_created_total (counter), payments_processed_total (counter by method), payroll_entries_total (counter by status), currency_sync_last_success_timestamp (gauge). 4. Expose GET /metrics endpoint returning Prometheus text format. 5. Implement GET /healthz (liveness): return 200 if process is running. 6. Implement GET /readyz (readiness): check PostgreSQL pool connectivity (sqlx::PgPool::acquire) and Redis PING; return 200 only if both succeed, 503 otherwise. 7. Create OpenAPI spec (openapi.yaml or use utoipa crate for auto-generation): document all endpoints with request/response schemas, error codes, authentication requirements. 8. Serve OpenAPI spec at GET /api/v1/openapi.json.

## Validation
/metrics returns valid Prometheus exposition format with HTTP request histograms and custom counters; /healthz returns 200; /readyz returns 200 when databases are up and 503 when PostgreSQL is down; /api/v1/openapi.json returns valid OpenAPI 3.0 spec that validates with swagger-cli; all endpoints are documented in the spec.