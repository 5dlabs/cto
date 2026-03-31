Implement subtask 4010: Add Prometheus metrics, health checks, and OpenAPI documentation

## Objective
Instrument the Finance service with Prometheus metrics, implement health/readiness endpoints, and generate OpenAPI specification.

## Steps
1. Add dependencies: axum-prometheus or metrics + metrics-exporter-prometheus, utoipa (for OpenAPI). 2. Create `src/handlers/health.rs`. GET /health — return 200 with {status: 'healthy'}. GET /ready — check PostgreSQL connectivity (SELECT 1), check Redis connectivity (PING), return 200 if both pass, 503 otherwise. 3. Add Prometheus metrics middleware to the Axum router. Track: http_requests_total (method, path, status), http_request_duration_seconds (histogram), active_connections gauge. 4. Add custom business metrics: invoices_created_total (counter), payments_processed_total (counter, by method), stripe_webhook_events_total (counter, by event_type), currency_sync_last_success (gauge timestamp), overdue_invoices_count (gauge). Instrument the relevant service functions. 5. Expose GET /metrics in Prometheus exposition format. 6. Add utoipa OpenAPI annotations to all handler functions. Generate the OpenAPI spec. Serve GET /api-docs for the JSON spec and optionally /swagger-ui using utoipa-swagger-ui. 7. Register /health, /ready, /metrics routes outside of versioned API prefix.

## Validation
GET /health returns 200 with healthy status. GET /ready returns 200 when DB and Redis are available, 503 when either is down. GET /metrics returns valid Prometheus exposition format with all custom metrics. GET /api-docs returns valid OpenAPI 3.0 JSON spec covering all endpoints. Business metrics increment correctly after invoicing/payment operations.