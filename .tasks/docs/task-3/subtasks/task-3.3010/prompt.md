Implement subtask 3010: Add health, metrics, and API documentation endpoints

## Objective
Implement Prometheus metrics collection, health/readiness probes, and generate API documentation for the RMS gRPC and REST endpoints.

## Steps
1. Add Prometheus Go client dependency. 2. Implement /healthz (liveness) and /readyz (readiness) endpoints that check PostgreSQL and Redis connectivity. 3. Register Prometheus metrics: request count, request duration histogram, error count — broken down by service and method. Add a /metrics endpoint. 4. Generate API documentation: use protoc-gen-openapiv2 to produce Swagger/OpenAPI spec from proto files with grpc-gateway annotations. 5. Optionally serve the Swagger JSON at /swagger.json or embed Swagger UI. 6. Verify all endpoints are accessible through both gRPC reflection and REST.

## Validation
GET /healthz returns 200 when dependencies are up; GET /readyz returns 503 when PostgreSQL is unreachable; GET /metrics returns Prometheus-formatted metrics with expected labels; OpenAPI spec is valid JSON and documents all REST endpoints.