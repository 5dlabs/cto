Implement subtask 3011: Expose REST endpoints via grpc-gateway and implement health/metrics

## Objective
Configure grpc-gateway to serve all five RMS services as REST endpoints, add Prometheus metrics instrumentation, and implement health/readiness endpoints.

## Steps
1. In main.go, register all five services' gRPC-gateway handlers on the HTTP mux. 2. Configure JSON marshaling with google.golang.org/protobuf/encoding/protojson (use snake_case, emit defaults). 3. Add Swagger/OpenAPI endpoint serving generated swagger.json from proto annotations. 4. Implement `/healthz` endpoint: check PostgreSQL connectivity, Redis connectivity, return overall health. 5. Implement `/readyz` endpoint: return ready only when all dependencies are connected and migrations are applied. 6. Add Prometheus metrics endpoint `/metrics` using prometheus/client_golang. 7. Add gRPC interceptors for: request logging (structured JSON), latency histogram, error rate counter. 8. Add HTTP middleware on the gateway for: request ID propagation, CORS headers, request logging. 9. Verify all REST routes match the PRD-specified paths (e.g., /api/v1/opportunities, /api/v1/projects, etc.).

## Validation
All REST endpoints are reachable via HTTP and return correct JSON responses; /healthz returns 200 when all deps are up; /readyz returns 503 until migrations complete; /metrics returns Prometheus-formatted metrics; Swagger JSON is served and valid; request IDs propagate through logs.