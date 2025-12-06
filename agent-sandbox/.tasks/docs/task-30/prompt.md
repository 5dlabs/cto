# Task 30: Implement Prometheus metrics and structured logging

## Role

You are a Senior Rust Engineer with expertise in systems programming and APIs implementing Task 30.

## Goal

Add observability with Prometheus metrics endpoint, structured JSON logging, and distributed tracing with trace IDs

## Requirements

1. Add dependencies: prometheus = "0.13", tracing = "0.1", tracing-subscriber = { version = "0.3", features = ["json"] }, uuid = "1.6"
2. Create infra/metrics.rs:
   - Register metrics: http_requests_total (counter), http_request_duration_seconds (histogram), websocket_connections (gauge), rate_limit_rejections_total (counter)
   - GET /metrics endpoint returning Prometheus format
3. Create api/middleware/metrics.rs:
   - Record http_requests_total with labels: method, path, status
   - Record http_request_duration_seconds histogram
4. Setup structured logging in main.rs:
   - tracing_subscriber::fmt().json().with_target(false).init()
   - Log format: {"timestamp", "level", "message", "trace_id", "span"}
5. Create api/middleware/tracing.rs:
   - Generate trace_id (UUID) for each request
   - Add to response header: X-Trace-Id
   - Include in all log entries via tracing span
6. Instrument key functions with #[tracing::instrument]
7. Log important events: auth attempts, rate limit hits, errors

## Acceptance Criteria

Verify /metrics endpoint returns valid Prometheus format, test metrics increment on requests, verify trace_id in logs and headers, load test to validate histogram buckets

## Constraints

- Match codebase patterns
- Create PR with atomic commits
- Include unit tests
- PR title: `feat(task-30): Implement Prometheus metrics and structured logging`
