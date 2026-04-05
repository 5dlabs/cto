Implement subtask 6008: Add Prometheus metrics and health endpoints

## Objective
Implement Prometheus metrics exposition and health/readiness probe endpoints for the social media engine.

## Steps
1. Install prom-client package.
2. Create `src/observability.ts` module.
3. Define and register metrics:
   - `social_requests_total` (counter, labels: endpoint, method, status)
   - `social_request_duration_seconds` (histogram, labels: endpoint)
   - `ai_api_calls_total` (counter, labels: provider, operation, status)
   - `ai_api_call_duration_seconds` (histogram, labels: provider, operation)
   - `publishing_attempts_total` (counter, labels: platform, status)
   - `approval_decisions_total` (counter, labels: decision)
   - `portfolio_syncs_total` (counter, labels: status)
4. Add Elysia middleware/plugin that records request metrics for all routes.
5. Instrument AI service calls and publishing service calls to record metrics.
6. Implement GET `/metrics` endpoint with prom-client default registry.
7. Implement GET `/healthz` — returns 200 if process is alive.
8. Implement GET `/readyz` — returns 200 if PostgreSQL and S3 connections are healthy, 503 otherwise.

## Validation
Verify /healthz returns 200. Verify /readyz returns 200 when DB and S3 are connected, 503 when either is down. Verify /metrics returns valid Prometheus format with expected metric names. Make API calls and verify counters increment.