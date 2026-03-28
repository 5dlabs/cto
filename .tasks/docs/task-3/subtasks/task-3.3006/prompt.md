Implement subtask 3006: Integrate grpc-gateway, Redis caching, and health checks

## Objective
Integrate `grpc-gateway` to expose REST endpoints, add Redis for session caching, and implement basic health checks for gRPC and REST endpoints.

## Steps
1. Integrate `grpc-gateway` to expose REST endpoints for all gRPC services.2. Integrate `go-redis` client for session caching, using the Redis URL from `sigma1-infra-endpoints`.3. Add `/healthz` and `/readyz` endpoints for both gRPC and REST, checking database and Redis connectivity.

## Validation
1. Use `curl` to verify REST endpoints exposed by `grpc-gateway` are functional.2. Verify Redis caching by setting and retrieving a session value.3. Access health check endpoints and confirm they return 200 OK.