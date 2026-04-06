Implement subtask 3001: Initialize Go project with gRPC and grpc-gateway scaffolding

## Objective
Set up the Go module, directory structure, gRPC server bootstrap, grpc-gateway reverse proxy, and ConfigMap-based configuration loading for the RMS service.

## Steps
1. Initialize Go module (e.g., `go mod init sigma1/rms`). 2. Set up directory structure: `/cmd/rms-server/`, `/internal/`, `/proto/`, `/pkg/`. 3. Add dependencies: google.golang.org/grpc, grpc-ecosystem/grpc-gateway/v2, lib/pq or pgx for PostgreSQL, go-redis/redis for Redis. 4. Create `cmd/rms-server/main.go` that boots a gRPC server and a grpc-gateway HTTP mux on separate ports. 5. Implement config loading from environment variables, expecting values injected via `envFrom: sigma1-infra-endpoints` ConfigMap (POSTGRES_HOST, POSTGRES_PORT, REDIS_HOST, etc.). 6. Add a Makefile with targets: `proto-gen`, `build`, `run`, `test`. 7. Set up `buf.yaml` or direct protoc invocation for proto generation.

## Validation
The server binary compiles without errors; `make build` succeeds; the gRPC server starts and listens on the configured port; the HTTP gateway starts and returns 404 for undefined routes; config values are read from environment variables.