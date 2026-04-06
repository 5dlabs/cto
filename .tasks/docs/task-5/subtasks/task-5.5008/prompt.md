Implement subtask 5008: Add Prometheus metrics and request/response schema validation

## Objective
Instrument the service with Prometheus metrics for vetting pipeline observability and ensure all input/output schemas are validated.

## Steps
1. Add prometheus and metrics dependencies (e.g., metrics, metrics-exporter-prometheus).
2. Define metrics:
   - `vetting_requests_total` (counter, labels: endpoint, status)
   - `vetting_duration_seconds` (histogram, labels: endpoint)
   - `vetting_external_api_duration_seconds` (histogram, labels: api_name)
   - `vetting_score_distribution` (histogram, labels: rating)
3. Add middleware to record request count and latency for all endpoints.
4. Instrument each external API call with timing metrics.
5. Expose GET `/metrics` endpoint in Prometheus text format.
6. Validate request bodies using serde with strict deserialization (deny_unknown_fields where appropriate).
7. Validate response bodies match expected schemas before sending.
8. Add structured logging with tracing crate for each vetting pipeline step.

## Validation
GET /metrics returns valid Prometheus text format with all defined metrics present; after running a vetting request, counters and histograms show non-zero values; malformed request bodies are rejected with 400 and descriptive error messages.