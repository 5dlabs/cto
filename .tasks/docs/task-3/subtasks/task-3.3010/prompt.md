Implement subtask 3010: Wire grpc-gateway REST endpoints, Prometheus metrics, and health checks

## Objective
Complete the REST gateway configuration, add Prometheus instrumentation to all gRPC handlers, and implement readiness/liveness health check endpoints.

## Steps
1. In cmd/server/main.go, register all five service handlers on the grpc-gateway mux. Verify all HTTP annotations produce working REST endpoints. 2. Add grpc-ecosystem/go-grpc-prometheus middleware to the gRPC server to instrument all RPCs with request count, latency histograms, and error rate metrics. 3. Expose /metrics endpoint on the REST gateway (or a separate admin port) for Prometheus scraping. 4. Implement /healthz (liveness) that returns 200 if the process is running. 5. Implement /readyz (readiness) that checks PostgreSQL and Redis connectivity and returns 200 only if both are reachable. 6. Add custom business metrics: opportunities_created_total, projects_active_gauge, inventory_checked_out_gauge, deliveries_in_transit_gauge. 7. Verify all REST endpoints match the grpc-gateway annotations (test a sample from each service).

## Validation
curl requests to REST endpoints (e.g., POST /api/v1/opportunities) return correct responses; /metrics returns Prometheus-formatted metrics including gRPC histograms and custom counters; /healthz returns 200; /readyz returns 503 when database is down and 200 when up.