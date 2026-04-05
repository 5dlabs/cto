Implement subtask 2003: Implement shared-observability crate with Prometheus metrics and structured logging

## Objective
Build the shared-observability crate providing Prometheus metrics integration for Axum via axum-prometheus and structured JSON logging via tracing + tracing-subscriber.

## Steps
1. Create `crates/shared-observability/Cargo.toml` depending on axum-prometheus, tracing, tracing-subscriber (features: json, env-filter), metrics, metrics-exporter-prometheus.
2. Implement `pub fn init_logging()` — initializes tracing-subscriber with JSON formatter, env-filter reading `RUST_LOG` (default `info`), timestamp in RFC3339.
3. Implement `pub fn metrics_layer() -> axum_prometheus::PrometheusMetricLayer` — returns the Axum layer that auto-instruments all routes with request_duration_seconds, request_count, etc.
4. Implement `pub fn metrics_handler() -> impl IntoResponse` — returns the `/metrics` Prometheus text exposition endpoint.
5. Export a `setup_metrics_route(router: Router) -> Router` helper that adds `GET /metrics` to any Axum router.
6. Add tracing::instrument re-export for convenient use in services.

## Validation
Unit test: init_logging() does not panic, metrics_layer() returns a valid layer. Integration test: create a minimal Axum app with the metrics layer, make a request, then GET /metrics and verify it contains `http_requests_total` or equivalent counter. Verify structured log output contains JSON with timestamp, level, message fields.