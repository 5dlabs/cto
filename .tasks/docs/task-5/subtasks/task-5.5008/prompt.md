Implement subtask 5008: Add Prometheus metrics and health endpoints

## Objective
Implement Prometheus metrics exposition and health/readiness probe endpoints for the vetting service.

## Steps
1. Add `metrics` and `metrics-exporter-prometheus` crates to dependencies.
2. Create `src/observability.rs` module.
3. Define and register metrics:
   - `vetting_requests_total` (counter, labels: endpoint, status)
   - `vetting_request_duration_seconds` (histogram, labels: endpoint)
   - `external_api_calls_total` (counter, labels: provider, status)
   - `external_api_call_duration_seconds` (histogram, labels: provider)
   - `vetting_scores_distribution` (histogram, labels: classification)
4. Add Axum middleware layer that records request metrics for all /api/v1/vetting/* routes.
5. Instrument each external API client to record call count and duration.
6. Implement GET `/metrics` endpoint exposing Prometheus text format.
7. Implement GET `/healthz` (liveness) — returns 200 if process is running.
8. Implement GET `/readyz` (readiness) — returns 200 if database connection is healthy, 503 otherwise.

## Validation
Verify /healthz returns 200. Verify /readyz returns 200 when DB is connected and 503 when disconnected. Verify /metrics returns valid Prometheus text format containing expected metric names. Make a vetting request and verify counters and histograms are incremented.