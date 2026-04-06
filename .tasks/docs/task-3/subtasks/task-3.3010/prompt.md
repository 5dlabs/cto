Implement subtask 3010: Add Prometheus metrics and health endpoints on dedicated port 8081

## Objective
Expose /metrics, /health/live, and /health/ready endpoints on a separate net/http mux listening on :8081 to avoid conflicts with the grpc-gateway mux on :8080.

## Steps
Create internal/health/server.go. Instantiate a new http.NewServeMux() (NOT the default mux). Register /health/live handler returning 200 OK with body {status: live}. Register /health/ready handler that pings the pgx pool (pool.Ping(ctx)) and the Redis client (client.Ping(ctx)); return 200 if both succeed, 503 otherwise with JSON indicating which dependency is down. Register /metrics handler using promhttp.Handler() from prometheus/client_golang. In main.go, spawn this mux in a separate goroutine on :8081. Define at least two custom Prometheus counters: rms_grpc_requests_total (labeled by method) and rms_inventory_conflicts_total. Increment rms_inventory_conflicts_total in the conflict detection path.

## Validation
GET http://localhost:8081/health/live returns 200. GET http://localhost:8081/health/ready returns 200 when postgres and redis are reachable, 503 when postgres is stopped. GET http://localhost:8081/metrics returns text/plain with rms_grpc_requests_total and rms_inventory_conflicts_total metric lines.