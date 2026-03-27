Implement task 4: RMS Service - Core gRPC & REST Gateway (Grizz - Go/gRPC)

## Goal
Develop the core Rental Management System (RMS) service, focusing on Opportunity and Project management. This includes defining protobuf schemas, implementing gRPC services, and exposing them via a RESTful grpc-gateway.

## Task Context
- Agent owner: Grizz
- Stack: Go/gRPC
- Priority: high
- Dependencies: 1

## Implementation Plan
1. Initialize a new Go 1.22+ gRPC project. 2. Define protobuf schemas for `OpportunityService` and `ProjectService`, including `CreateOpportunityRequest`, `GetOpportunityRequest`, `UpdateOpportunityRequest`, `ListOpportunitiesRequest`, `ScoreLeadRequest`, `CreateProjectRequest`, `GetProjectRequest`, `UpdateProjectRequest`. 3. Generate Go code from protobuf definitions. 4. Implement the gRPC server for `OpportunityService` and `ProjectService`, including methods for creating, retrieving, updating, and listing opportunities and projects. Implement `ScoreLead` as a placeholder. 5. Define the PostgreSQL schema for `Opportunity` and `Project` data models, including `ID`, `CustomerID`, `Status`, `EventDateStart`, `EventDateEnd`, `Venue`, `TotalEstimate`, `LeadScore`, `Notes` for Opportunity, and `ID`, `OpportunityID`, `CustomerID`, `Status`, `ConfirmedAt`, `EventDates`, `VenueAddress`, `CrewNotes` for Project. Use `gorm` or `sqlc` for database interactions. 6. Set up `grpc-gateway` to expose REST endpoints for opportunities and projects (e.g., `POST /api/v1/opportunities`, `GET /api/v1/opportunities/:id`, `PATCH /api/v1/opportunities/:id`, `POST /api/v1/opportunities/:id/approve`, `POST /api/v1/opportunities/:id/convert`, `GET /api/v1/projects`, `GET /api/v1/projects/:id`). 7. Configure the service to connect to PostgreSQL using credentials from the 'sigma1-infra-endpoints' ConfigMap.

## Acceptance Criteria
1. Deploy the service and verify it starts successfully, connecting to PostgreSQL. 2. Use `grpcurl` to test gRPC endpoints for creating, retrieving, updating, and listing opportunities and projects. 3. Use `curl` or Postman to test the REST endpoints exposed by `grpc-gateway` for the same operations. 4. Verify that data persisted via gRPC is correctly retrieved via REST and vice-versa. 5. Confirm `ScoreLead` returns a valid `LeadScore` (even if placeholder logic). 6. Run `go test ./...` and `go vet ./...` to ensure code quality and correctness. 7. Verify database schema matches protobuf definitions.

## Subtasks
- Implement RMS Service - Core gRPC & REST Gateway (Grizz - Go/gRPC): Develop the core Rental Management System (RMS) service, focusing on Opportunity and Project management. This includes defining protobuf schemas, implementing gRPC services, and exposing them via a RESTful grpc-gateway.

## Deliverables
- Update the relevant code, configuration, and tests.
- Keep artifacts aligned with the acceptance criteria.
- Document blockers or assumptions in your final summary.