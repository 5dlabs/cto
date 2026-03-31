Implement subtask 3012: Register all services with grpc-gateway and expose REST endpoints

## Objective
Wire all five gRPC services into the grpc-gateway HTTP mux so that REST endpoints are automatically exposed based on proto annotations.

## Steps
1. In `main.go`, register all five service handlers with the gRPC server.
2. Create the grpc-gateway runtime mux and register all five service handlers for HTTP:
   - RegisterOpportunityServiceHandlerFromEndpoint
   - RegisterProjectServiceHandlerFromEndpoint
   - RegisterInventoryServiceHandlerFromEndpoint
   - RegisterCrewServiceHandlerFromEndpoint
   - RegisterDeliveryServiceHandlerFromEndpoint
3. Configure JSON marshaler with `runtime.WithMarshalerOption` for camelCase field names and enum string values.
4. Add CORS middleware for the HTTP gateway if needed.
5. Verify all REST endpoints respond correctly by starting the server and making HTTP requests.

## Validation
Start the server and verify all REST endpoints respond: POST/GET /api/v1/opportunities, POST/GET /api/v1/projects, POST/GET /api/v1/inventory, GET /api/v1/inventory/barcode/{barcode}, POST/GET /api/v1/crew, POST/GET /api/v1/deliveries. JSON responses use camelCase. gRPC reflection is enabled for grpcurl testing.