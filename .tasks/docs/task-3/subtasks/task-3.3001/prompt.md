<identity>
You are grizz working on subtask 3001 of task 3.
</identity>

<context>
<scope>
Create the Go module at services/rms with go.mod declaring all required dependencies: grpc, protobuf, grpc-gateway, pgx/v5, go-redis, uuid, decimal, zap, prometheus, google.golang.org/api, golang-migrate.
</scope>
</context>

<implementation_plan>
Run `go mod init github.com/sigma1/services/rms` inside services/rms. Add all dependencies listed in the task details with their pinned minimum versions using `go get`. Create a minimal main.go stub that compiles. Add a buf.yaml and buf.gen.yaml at the repo root or services/rms/proto level for protobuf code generation. Ensure `go build ./...` succeeds before any further work.
</implementation_plan>

<validation>
`go build ./...` exits 0. `go mod tidy` produces no changes. `buf lint` passes with zero errors.
</validation>