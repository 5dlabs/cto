Implement subtask 2008: Add Prometheus metrics and health check endpoints

## Objective
Implement /health, /ready, and /metrics endpoints for Kubernetes probes and Prometheus scraping.

## Steps
1. Add the prometheus and axum-prometheus or metrics crate to Cargo.toml. 2. Create src/handlers/health.rs module. 3. Implement GET /health: return 200 { status: 'ok' } (liveness probe — no dependency checks). 4. Implement GET /ready: check PostgreSQL connectivity (SELECT 1) and Redis connectivity (PING), return 200 if both pass, 503 if any fail. Include details: { postgres: 'ok'/'error', redis: 'ok'/'error' }. 5. Implement GET /metrics: expose Prometheus-format metrics including: http_requests_total (by method, path, status), http_request_duration_seconds (histogram), db_pool_connections_active, redis_cache_hits_total, redis_cache_misses_total. 6. Add an Axum middleware layer that records request count and duration for every request. 7. Register these routes outside the /api/v1 prefix (at root level).

## Validation
GET /health returns 200; GET /ready returns 200 when DB and Redis are up, 503 when either is down; GET /metrics returns valid Prometheus text format with expected metric names; request counter increments after making API calls.