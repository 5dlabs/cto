Implement subtask 3001: Initialize Go module and configure buf for protobuf code generation

## Objective
Set up the Go module `github.com/5dlabs/sigma1-rms` with Go 1.22+, configure buf.yaml and buf.gen.yaml for protoc-gen-go, protoc-gen-go-grpc, and protoc-gen-grpc-gateway code generation. Establish the project directory structure including proto/, cmd/, internal/, migrations/, and deploy/ directories.

## Steps
1. Run `go mod init github.com/5dlabs/sigma1-rms` with Go 1.22+.
2. Create directory structure: `proto/sigma1/rms/v1/`, `cmd/rms-server/`, `internal/service/`, `internal/db/`, `internal/middleware/`, `migrations/`, `deploy/`.
3. Install buf CLI and create `buf.yaml` at project root with `lint` and `breaking` configuration.
4. Create `buf.gen.yaml` with plugins: `protoc-gen-go` (paths=source_relative), `protoc-gen-go-grpc` (paths=source_relative), `protoc-gen-grpc-gateway` (paths=source_relative, generate_unbound_methods=true).
5. Add `go.sum` dependencies: `google.golang.org/grpc`, `google.golang.org/protobuf`, `github.com/grpc-ecosystem/grpc-gateway/v2`.
6. Add googleapis proto dependencies in `buf.yaml` deps for `google/api/annotations.proto` and `google/api/http.proto`.
7. Verify `buf build` succeeds with empty proto directory.
8. Create a basic `cmd/rms-server/main.go` skeleton that imports the generated package paths (placeholder).

## Validation
Verify `buf build` completes without errors. Verify `go build ./...` succeeds. Confirm directory structure matches expected layout.