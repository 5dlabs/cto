Implement subtask 3001: Scaffold Go project with gRPC server and grpc-gateway

## Objective
Initialize the Go module, set up the gRPC server entrypoint, configure grpc-gateway for REST proxy, and establish the project directory structure following idiomatic Go layout for a multi-service gRPC application.

## Steps
1. Run `go mod init` for the RMS service module.
2. Create directory structure: `/cmd/rms-server/`, `/internal/`, `/proto/`, `/pkg/`, `/migrations/`.
3. Set up a `main.go` in `/cmd/rms-server/` that initializes a gRPC server on a configurable port (default 50051).
4. Add grpc-gateway runtime mux and register it on an HTTP port (default 8080).
5. Configure graceful shutdown with signal handling (SIGTERM, SIGINT).
6. Add a `buf.yaml` and `buf.gen.yaml` for protobuf tooling with Go and grpc-gateway plugins.
7. Include a `Makefile` with targets: `proto-gen`, `build`, `run`, `test`.
8. Add a Dockerfile with multi-stage build (builder + distroless runtime).

## Validation
The gRPC server starts and listens on port 50051. The HTTP gateway starts and listens on port 8080. `make build` produces a binary without errors. `make proto-gen` completes successfully (even with empty proto files). Dockerfile builds successfully.