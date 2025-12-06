# Task 14: Create observability infrastructure with Prometheus and structured logging

## Role

You are a Senior Rust Engineer with expertise in systems programming and APIs implementing Task 14.

## Goal

Implement metrics endpoint, structured JSON logging with trace IDs, and health check endpoints for production monitoring

## Requirements

1. Add dependencies: tracing = "0.1", tracing-subscriber = { version = "0.3", features = ["json"] }, metrics = "0.21", metrics-exporter-prometheus = "0.13"
2. Initialize tracing subscriber in main.rs:
   - JSON formatter with trace_id, span_id, level, message, timestamp
   - Filter by RUST_LOG env var
3. Create src/api/observability.rs:
   - GET /metrics: expose Prometheus metrics (request count, duration histogram, active connections)
   - GET /health/live: liveness probe (return 200 if server running)
   - GET /health/ready: readiness probe (check DB and Redis connectivity)
4. Add middleware to track metrics:
   - HTTP request counter by method, path, status
   - Request duration histogram
   - Active WebSocket connections gauge
5. Log key events: auth success/failure, task operations, errors with context
6. Generate trace_id per request, propagate through call chain

## Acceptance Criteria

Unit test health check logic. Integration test: make requests, verify metrics incremented. Test /health/ready returns 503 when DB unavailable. Verify JSON logs parseable and contain trace_id

## Constraints

- Match codebase patterns
- Create PR with atomic commits
- Include unit tests
- PR title: `feat(task-14): Create observability infrastructure with Prometheus and structured logging`
