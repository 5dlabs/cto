Implement subtask 3001: Scaffold Go project with gRPC, grpc-gateway, and infrastructure connectivity

## Objective
Initialize the Go module with gRPC server, grpc-gateway HTTP proxy, PostgreSQL connection pool, and Redis client, all reading connection strings from the sigma1-infra-endpoints ConfigMap. Include health check and readiness endpoints.

## Steps
1. Create Go module (go mod init) with Go 1.22+.
2. Add dependencies: google.golang.org/grpc, grpc-ecosystem/grpc-gateway/v2, pgx/v5 for PostgreSQL, go-redis/redis/v9.
3. Set up main.go with a gRPC server on one port and grpc-gateway HTTP reverse proxy on another.
4. Read POSTGRES_URL and REDIS_URL from environment variables injected via envFrom referencing 'sigma1-infra-endpoints' ConfigMap.
5. Initialize a pgxpool.Pool for PostgreSQL and redis.Client for Redis at startup, with connection validation.
6. Register /healthz (liveness) and /readyz (readiness) HTTP endpoints that verify DB and Redis connectivity.
7. Set up structured logging (slog or zerolog).
8. Create a Dockerfile with multi-stage build for minimal production image.
9. Add a Makefile with targets: proto-gen, build, test, lint.

## Validation
Application starts successfully, /healthz returns 200, /readyz returns 200 when DB and Redis are reachable and 503 when either is down. go build completes without errors. Docker image builds successfully.