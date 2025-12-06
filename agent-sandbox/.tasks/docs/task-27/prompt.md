# Task 27: Setup observability with Prometheus metrics and structured logging

## Role

You are a Senior DevOps Engineer with expertise in Kubernetes, GitOps, and CI/CD implementing Task 27.

## Goal

Implement Prometheus metrics endpoint, structured JSON logging with trace IDs, and health check endpoints for Kubernetes.

## Requirements

1. Add dependencies:
   - prometheus = "0.13"
   - tracing = "0.1"
   - tracing-subscriber = { version = "0.3", features = ["json", "env-filter"] }
   - uuid = { version = "1", features = ["v4"] }
2. Create infra/metrics.rs:
   - Register metrics: http_requests_total (counter), http_request_duration_seconds (histogram), active_websocket_connections (gauge), task_operations_total (counter)
   - Middleware to record metrics on each request
3. Implement infra/tracing.rs:
   - Initialize tracing_subscriber with JSON formatter
   - Generate trace_id (UUID v4) per request, inject into logs
   - Log level from RUST_LOG env var
4. Add endpoints in api/health.rs:
   - GET /health/live -> 200 OK (liveness probe)
   - GET /health/ready -> checks DB and Redis connections, 200 if healthy
   - GET /metrics -> Prometheus text format
5. Add trace_id to response headers: X-Trace-Id
6. Log format: {"timestamp", "level", "trace_id", "message", "fields"}
7. Instrument key functions with tracing spans

## Acceptance Criteria

Unit tests for metrics recording. Integration tests: make requests, verify metrics incremented. Check /health/ready returns 503 when DB unavailable. Verify logs are valid JSON with trace_ids. Load test and verify metrics in Prometheus. Test log filtering with RUST_LOG.

## Constraints

- Match codebase patterns
- Create PR with atomic commits
- Include unit tests
- PR title: `feat(task-27): Setup observability with Prometheus metrics and structured logging`
