Implement subtask 3001: Scaffold Go project with gRPC, grpc-gateway, and infrastructure config

## Objective
Initialize a Go 1.22+ module with gRPC server, grpc-gateway HTTP proxy, project directory structure, and infrastructure configuration referencing the sigma1-infra-endpoints ConfigMap.

## Steps
1. Run `go mod init` for the RMS service module targeting Go 1.22+.
2. Add dependencies: google.golang.org/grpc, grpc-ecosystem/grpc-gateway/v2, lib/pq or pgx for PostgreSQL, go-redis/redis for session cache.
3. Create directory structure: /cmd/rms-server (main entrypoint), /internal/server (gRPC server setup), /internal/gateway (grpc-gateway setup), /proto (protobuf source), /gen (generated code), /internal/db (database layer), /internal/config.
4. Implement config package that reads environment variables injected via envFrom from 'sigma1-infra-endpoints' ConfigMap (POSTGRES_URL, REDIS_URL, etc.).
5. Set up main.go to boot gRPC server on one port and grpc-gateway HTTP proxy on another.
6. Add a health check endpoint (GET /healthz) on the gateway.
7. Set up Makefile or Taskfile with targets: proto-gen, build, test, run.
8. Add buf.yaml or direct protoc configuration for protobuf code generation.

## Validation
Service starts without errors when POSTGRES_URL and REDIS_URL env vars are set; /healthz returns 200 OK; gRPC reflection lists no services yet but server responds to grpc.health.v1.Health.