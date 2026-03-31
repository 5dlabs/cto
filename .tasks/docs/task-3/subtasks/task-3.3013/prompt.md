Implement subtask 3013: Add Prometheus metrics and health check endpoints

## Objective
Instrument the RMS service with Prometheus metrics for request counts, latencies, and error rates, and add gRPC health check and HTTP readiness/liveness probes.

## Steps
1. Add `go.opentelemetry.io/contrib/instrumentation/google.golang.org/grpc/otelgrpc` or `grpc-ecosystem/go-grpc-middleware/v2` Prometheus interceptors.
2. Register unary and stream server interceptors that track:
   - `grpc_server_handled_total` (counter by method, code)
   - `grpc_server_handling_seconds` (histogram by method)
3. Expose `/metrics` endpoint on the HTTP port using `promhttp.Handler()`.
4. Implement gRPC Health Check protocol (`grpc.health.v1.Health`) that checks PostgreSQL and Redis connectivity.
5. Add HTTP endpoints:
   - `GET /healthz` — liveness (always 200 if process is running)
   - `GET /readyz` — readiness (200 only if DB and Redis are reachable)
6. Register health and metrics in `main.go`.

## Validation
GET /metrics returns Prometheus-format metrics including grpc_server_handled_total. GET /healthz returns 200. GET /readyz returns 200 when DB and Redis are up, 503 when either is down. grpcurl to grpc.health.v1.Health/Check returns SERVING.