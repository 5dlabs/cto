Implement subtask 5007: Add Prometheus metrics, health endpoints, and OpenAPI documentation

## Objective
Instrument the vetting service with Prometheus metrics for observability, add health/readiness endpoints, and generate OpenAPI documentation for all endpoints.

## Steps
1. Add metrics using `metrics` and `metrics-exporter-prometheus` crates. 2. Track: vetting_pipeline_duration_seconds (histogram), vetting_pipeline_total (counter by status: success/partial/failure), vetting_source_duration_seconds (histogram by source: opencorporates/linkedin/google/credit), vetting_source_errors_total (counter by source and error type), vetting_score_distribution (histogram of final scores). 3. Expose /metrics endpoint for Prometheus scraping. 4. Implement /health/live (liveness: always 200 if process running) and /health/ready (readiness: checks PostgreSQL connectivity). 5. Use utoipa to annotate all endpoint handlers and models with OpenAPI metadata. 6. Serve the generated OpenAPI JSON at /api/v1/vetting/openapi.json. 7. Verify all endpoints are documented with request/response schemas, error codes, and descriptions.

## Validation
GET /metrics returns Prometheus-formatted metrics including vetting-specific counters and histograms. GET /health/live returns 200. GET /health/ready returns 200 when DB is connected, 503 when DB is unavailable. GET /api/v1/vetting/openapi.json returns valid OpenAPI 3.0 JSON that documents all three vetting endpoints with correct schemas.