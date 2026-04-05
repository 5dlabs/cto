Implement subtask 3009: Add Prometheus metrics and health endpoints

## Objective
Instrument all RMS services with Prometheus metrics (request counts, latencies, error rates) and expose health/readiness probe endpoints.

## Steps
1. Add prometheus/client_golang dependency.
2. Create `internal/metrics/` package with middleware interceptors for gRPC (UnaryServerInterceptor, StreamServerInterceptor) that record: rms_grpc_requests_total (method, status), rms_grpc_request_duration_seconds (method), rms_grpc_errors_total (method, code).
3. Add HTTP middleware for grpc-gateway endpoints with equivalent metrics.
4. Add business metrics: rms_opportunities_created_total, rms_projects_active, rms_inventory_checked_out, rms_deliveries_scheduled.
5. Expose /metrics endpoint on the HTTP port for Prometheus scraping.
6. Implement /healthz (liveness) endpoint: returns 200 if process is running.
7. Implement /readyz (readiness) endpoint: checks PostgreSQL connectivity and Redis connectivity, returns 200 only if both are reachable.
8. Register interceptors in the gRPC server setup in main.go.

## Validation
Verify /healthz returns 200. Verify /readyz returns 200 when DB and Redis are up, and 503 when either is down. Make several gRPC/REST calls and verify /metrics returns Prometheus-formatted output with correct counter increments and histogram buckets.