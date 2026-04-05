Implement subtask 3001: Initialize Go module and buf protobuf toolchain

## Objective
Set up the Go module, directory structure, and buf configuration for protobuf management and code generation across all 5 services.

## Steps
1. Run `go mod init github.com/sigma1/rms` with Go 1.22+.
2. Create directory structure: `proto/`, `internal/`, `cmd/server/`, `db/migrations/`.
3. Install and configure buf: create `buf.yaml` with `name: buf.build/sigma1/rms` and lint/breaking rules.
4. Create `buf.gen.yaml` with plugins: `protoc-gen-go`, `protoc-gen-go-grpc`, `protoc-gen-grpc-gateway`, `protoc-gen-openapiv2`.
5. Add `google/api/annotations.proto` and `google/api/http.proto` dependencies via buf BSR.
6. Create a minimal `proto/rms/v1/common.proto` with shared message types: `Timestamp`, `Money`, `Address`, `PaginationRequest`, `PaginationResponse`, `OrgId`.
7. Run `buf generate` to verify toolchain produces Go code in `gen/` directory.
8. Add Makefile targets: `proto-gen`, `proto-lint`, `proto-breaking`.

## Validation
Run `buf lint` with zero errors. Run `buf generate` and verify Go files are generated in the expected output directory. Verify `go build ./...` succeeds with generated code.