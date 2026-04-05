Implement subtask 3004: Generate Go code from protobuf definitions

## Objective
Configure protoc/buf for Go code generation including gRPC stubs, grpc-gateway reverse proxies, and OpenAPI specs from the .proto files.

## Steps
1. Create a buf.gen.yaml (or Makefile with protoc commands) that generates: Go protobuf messages (protoc-gen-go), gRPC service stubs (protoc-gen-go-grpc), grpc-gateway reverse proxy (protoc-gen-grpc-gateway), and optionally OpenAPI v2 spec (protoc-gen-openapiv2). 2. Run code generation and verify output lands in /internal/gen or /pkg/pb. 3. Ensure generated gateway code registers all five services on the gateway mux. 4. Add a `go generate` or Makefile target for repeatable generation. 5. Verify generated Go code compiles with `go build ./...`.

## Validation
Running `make proto` or `buf generate` produces Go files for all 5 services; `go build ./...` succeeds with generated code; gateway registration code exists for all services.