Implement subtask 3012: Implement health checks, Prometheus metrics, and structured logging

## Objective
Add gRPC health checking protocol, REST health endpoints, Prometheus metrics via grpc-prometheus, and structured logging with slog throughout the service.

## Steps
1. Health checks in `internal/health/health.go`:
   - Implement `grpc.health.v1.Health` service: `Check` and `Watch` RPCs.
   - Liveness: always SERVING if process is running.
   - Readiness: check pgx pool connectivity (`pool.Ping(ctx)`), return NOT_SERVING if DB unreachable.
   - Register on gRPC server.
2. REST health endpoints on grpc-gateway mux:
   - `GET /health/live` → 200 if process running.
   - `GET /health/ready` → 200 if DB connected, 503 otherwise.
3. Prometheus metrics:
   - Add `grpc-prometheus` server interceptors (unary + stream) to gRPC server.
   - Initialize `grpc_prometheus.EnableHandlingTimeHistogram()`.
   - Mount `/metrics` endpoint on HTTP mux using `promhttp.Handler()`.
   - Add custom business metrics: `rms_opportunities_converted_total`, `rms_conflict_detections_total`, `rms_gdpr_deletions_total` as Prometheus counters.
4. Structured logging:
   - Configure `slog.NewJSONHandler` as default logger in `cmd/server/main.go`.
   - Add `slog` logging to all service methods: log request start (with org_id, method), log errors, log completion with duration.
   - Add gRPC logging interceptor using `slog` for request/response metadata.
   - Include request_id in log context (extract from gRPC metadata or generate UUID).

## Validation
1) Health: start server, call gRPC Health.Check → SERVING. Call /health/live → 200. Call /health/ready with DB up → 200. Stop DB container, call /health/ready → 503. 2) Metrics: send 5 gRPC requests, scrape /metrics, verify grpc_server_handled_total counter = 5. Convert an opportunity, verify rms_opportunities_converted_total = 1. 3) Logging: capture log output during a request, verify JSON format with org_id, method, duration fields.