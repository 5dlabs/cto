Implement subtask 3013: Implement structured logging, Prometheus metrics, and health check endpoints

## Objective
Add zerolog structured JSON logging interceptor, grpc_prometheus metrics interceptors, custom HTTP metrics, and gRPC+HTTP health check endpoints.

## Steps
1. Add dependencies: `github.com/rs/zerolog`, `github.com/grpc-ecosystem/go-grpc-prometheus`, `google.golang.org/grpc/health`.
2. Create `internal/middleware/logging.go`:
   - Unary interceptor that logs each RPC call with: method, duration_ms, status_code, user_id (from context), request_id (generated UUID)
   - Use zerolog with JSON output to stdout
   - Log at Info level for success, Warn for client errors (4xx equivalent), Error for server errors
3. Create `internal/middleware/metrics.go`:
   - Initialize `grpc_prometheus.NewServerMetrics()` and register with default Prometheus registry
   - Enable handling time histogram
   - Add custom counters: `rms_opportunities_created_total`, `rms_projects_created_total`, `rms_inventory_transactions_total` (labeled by type)
4. Health checks:
   - Register gRPC health service (`grpc_health_v1.RegisterHealthServer`) that checks database connectivity via pgxpool.Ping()
   - Add HTTP endpoints on the gateway port: `/health/live` (always 200), `/health/ready` (200 if DB connected, 503 if pool exhausted or ping fails)
5. Expose `/metrics` endpoint on the HTTP server (port 8080) using `promhttp.Handler()`.
6. Wire all interceptors into the gRPC server in main.go: logging → metrics → auth → RBAC (in order).

## Validation
Verify `/metrics` includes `grpc_server_handled_total` counter after making RPC calls. Verify `/health/ready` returns 200 with active DB and 503 when connection is unavailable. Verify structured log output contains method, duration_ms, and status_code fields. Verify custom `rms_opportunities_created_total` counter increments after creating an opportunity.