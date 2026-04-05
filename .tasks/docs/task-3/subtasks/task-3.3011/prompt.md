Implement subtask 3011: Configure grpc-gateway HTTP server with REST endpoint mapping

## Objective
Set up the grpc-gateway reverse proxy HTTP server on port 8080 that translates REST calls to gRPC calls on port 8081, with proper JSON serialization options.

## Steps
1. Create `internal/gateway/gateway.go` that sets up the grpc-gateway runtime mux.
2. Configure `runtime.ServeMux` options:
   - `runtime.WithMarshalerOption` for JSON with `EmitUnpopulated: false`, `UseProtoNames: true` (snake_case field names)
   - `runtime.WithIncomingHeaderMatcher` to forward Authorization header to gRPC metadata
   - `runtime.WithErrorHandler` for custom error response format matching PRD expectations
3. Register all 5 service handlers with the gateway mux:
   - `RegisterOpportunityServiceHandlerFromEndpoint`
   - `RegisterProjectServiceHandlerFromEndpoint`
   - `RegisterInventoryServiceHandlerFromEndpoint`
   - `RegisterCrewServiceHandlerFromEndpoint`
   - `RegisterDeliveryServiceHandlerFromEndpoint`
4. Endpoint points to localhost:8081 (gRPC server) with insecure credentials (in-process communication).
5. Serve HTTP on port 8080 using `http.ListenAndServe`.
6. In `cmd/rms-server/main.go`, start both gRPC server (port 8081) and gateway HTTP server (port 8080) concurrently using goroutines with proper signal handling (SIGTERM, SIGINT) for graceful shutdown.
7. Add a CORS middleware if needed for browser-based MCP tool access (configurable via env var).

## Validation
Test `curl localhost:8080/api/v1/opportunities` returns valid JSON with proper snake_case field names. Test that POST requests with JSON bodies are correctly translated to gRPC. Test that Authorization header is forwarded to gRPC service. Test graceful shutdown: send SIGTERM and verify in-flight requests complete.