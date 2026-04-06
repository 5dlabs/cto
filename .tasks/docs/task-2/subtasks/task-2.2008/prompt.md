Implement subtask 2008: Implement Prometheus metrics middleware and /metrics endpoint

## Objective
Register a request counter and request duration histogram with Prometheus, instrument all HTTP handlers via a tower middleware layer, and expose GET /metrics in Prometheus text format.

## Steps
Create src/metrics.rs. Use prometheus crate. Register with lazy_static or once_cell: HTTP_REQUESTS_TOTAL: CounterVec with labels [method, route, status]. HTTP_REQUEST_DURATION_SECONDS: HistogramVec with labels [method, route] and buckets [0.005, 0.01, 0.025, 0.05, 0.1, 0.25, 0.5, 1.0, 2.5]. Create a MetricsLayer (tower middleware): record start time before inner service call, after response extract status code and route template (use MatchedPath extractor), increment counter and observe histogram. Create handler fn metrics_handler(): call prometheus::gather(), encode to text format with TextEncoder, return Response with content-type text/plain; version=0.0.4. Add GET /metrics route to Router (outside the rate-limited group). Wire MetricsLayer onto all routes in main.rs.

## Validation
GET /metrics returns HTTP 200 with Content-Type: text/plain; version=0.0.4. Response body contains lines matching: http_requests_total{method="GET",route="/api/v1/catalog/products",status="200"} and http_request_duration_seconds_bucket. After 10 GET /api/v1/catalog/products requests, http_requests_total counter value >= 10.