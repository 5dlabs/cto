Implement subtask 2003: Implement shared Prometheus metrics middleware

## Objective
Add request metrics middleware to the shared crate using the metrics and metrics-exporter-prometheus crates, exposed at GET /metrics.

## Steps
1. Add `metrics 0.22`, `metrics-exporter-prometheus 0.13` to shared Cargo.toml.
2. In `shared/src/metrics.rs`, implement setup function `pub fn init_metrics() -> PrometheusHandle` that installs the Prometheus recorder.
3. Create an Axum middleware layer `pub fn metrics_layer() -> impl Layer` that records:
   - `http_requests_total` counter with labels: method, path, status_code
   - `http_request_duration_seconds` histogram with labels: method, path
4. Implement `pub async fn metrics_handler(State(handle): State<PrometheusHandle>) -> impl IntoResponse` that renders Prometheus text format.
5. Export convenience function `pub fn metrics_route(handle: PrometheusHandle) -> Router` mounting at `GET /metrics`.

## Validation
Unit test that metrics_layer records counter increments. Integration test: send a request through a test Axum app with metrics middleware, then GET /metrics and verify http_requests_total and http_request_duration_seconds appear in the response body with expected labels.