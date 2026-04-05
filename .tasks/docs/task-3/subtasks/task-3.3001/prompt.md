Implement subtask 3001: Initialize Go project with gRPC and grpc-gateway scaffolding

## Objective
Set up the Go 1.22+ module with gRPC server, grpc-gateway reverse proxy, project directory structure, build tooling, and base configuration loading from ConfigMap environment variables.

## Steps
1. Initialize Go module (go mod init). 2. Add dependencies: google.golang.org/grpc, grpc-ecosystem/grpc-gateway/v2, protoc-gen-go, protoc-gen-go-grpc, protoc-gen-grpc-gateway. 3. Create project structure: /cmd/server (main entrypoint), /internal (business logic), /proto (protobuf definitions), /pkg (shared utilities), /migrations (SQL). 4. Implement main.go that starts a gRPC server and grpc-gateway HTTP mux on separate ports. 5. Add configuration loading via envFrom referencing the infra-endpoints ConfigMap (PostgreSQL connection string, Redis URL, etc.). 6. Add a Makefile with targets for protoc generation, build, and test. 7. Create a Dockerfile for containerized builds.

## Validation
Server starts successfully and listens on both gRPC and HTTP ports; `grpc_health_v1` check returns SERVING; HTTP gateway returns 404 for undefined routes (not connection errors).