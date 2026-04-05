Implement subtask 3001: Initialize Go project with gRPC, grpc-gateway, and database connectivity

## Objective
Set up the Go module, import gRPC and grpc-gateway dependencies, configure PostgreSQL and Redis connections using environment variables from the infra ConfigMap, and establish the project directory structure for all five services.

## Steps
1. Run `go mod init` for the RMS service module. 2. Add dependencies: google.golang.org/grpc, grpc-ecosystem/grpc-gateway/v2, pgx/v5 for PostgreSQL, go-redis/redis for Redis/Valkey. 3. Create directory structure: /proto, /internal/opportunity, /internal/project, /internal/inventory, /internal/crew, /internal/delivery, /internal/db, /cmd/server. 4. Implement a database connection pool in /internal/db that reads POSTGRES_URL and REDIS_URL from environment (sourced via envFrom on the infra-endpoints ConfigMap). 5. Create main.go in /cmd/server with a gRPC server and grpc-gateway mux wired together, listening on separate ports (e.g., :50051 for gRPC, :8080 for REST). 6. Add a basic health check endpoint on the REST gateway. 7. Verify the service starts and connects to PostgreSQL and Redis.

## Validation
Service binary compiles without errors; `go run cmd/server/main.go` starts and logs successful PostgreSQL and Redis connections; health endpoint at /healthz returns 200.