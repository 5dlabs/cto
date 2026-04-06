Implement subtask 3006: Implement OpportunityService and ProjectService gRPC handlers

## Objective
Implement the gRPC server handlers for OpportunityService and ProjectService, wiring protobuf RPCs to the repository layer with proper validation and error handling.

## Steps
1. Create internal/service/opportunity_service.go implementing the generated OpportunityServiceServer interface.
2. Wire each RPC to the repository layer: CreateOpportunity validates input, calls repo.Create, returns created entity; GetOpportunity calls repo.Get; ListOpportunities supports pagination/filtering; UpdateOpportunity validates and calls repo.Update; DeleteOpportunity calls repo.Delete (soft delete).
3. Create internal/service/project_service.go implementing ProjectServiceServer.
4. Implement all Project RPCs similarly, ensuring project creation validates the linked opportunity exists.
5. Add input validation using protovalidate or manual checks (required fields, valid enums, date ranges).
6. Map repository errors to appropriate gRPC status codes (NotFound, InvalidArgument, AlreadyExists, Internal).
7. Register both services with the gRPC server in main.go.

## Validation
Unit tests with mocked repositories verify correct behavior for each RPC; gRPC client calls return expected responses for happy paths; invalid inputs return InvalidArgument; missing entities return NotFound; all RPCs are accessible via both gRPC and REST (grpc-gateway).