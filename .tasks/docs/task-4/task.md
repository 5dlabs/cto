## RMS Service - Core gRPC & REST Gateway (Grizz - Go/gRPC)

### Objective
Develop the core Rental Management System (RMS) service, focusing on Opportunity and Project management. This includes defining protobuf schemas, implementing gRPC services, and exposing them via a RESTful grpc-gateway.

### Ownership
- Agent: Grizz
- Stack: Go/gRPC
- Priority: high
- Status: pending
- Dependencies: 1

### Implementation Details
1. Initialize a new Go 1.22+ gRPC project. 2. Define protobuf schemas for `OpportunityService` and `ProjectService`, including `CreateOpportunityRequest`, `GetOpportunityRequest`, `UpdateOpportunityRequest`, `ListOpportunitiesRequest`, `ScoreLeadRequest`, `CreateProjectRequest`, `GetProjectRequest`, `UpdateProjectRequest`. 3. Generate Go code from protobuf definitions. 4. Implement the gRPC server for `OpportunityService` and `ProjectService`, including methods for creating, retrieving, updating, and listing opportunities and projects. Implement `ScoreLead` as a placeholder. 5. Define the PostgreSQL schema for `Opportunity` and `Project` data models, including `ID`, `CustomerID`, `Status`, `EventDateStart`, `EventDateEnd`, `Venue`, `TotalEstimate`, `LeadScore`, `Notes` for Opportunity, and `ID`, `OpportunityID`, `CustomerID`, `Status`, `ConfirmedAt`, `EventDates`, `VenueAddress`, `CrewNotes` for Project. Use `gorm` or `sqlc` for database interactions. 6. Set up `grpc-gateway` to expose REST endpoints for opportunities and projects (e.g., `POST /api/v1/opportunities`, `GET /api/v1/opportunities/:id`, `PATCH /api/v1/opportunities/:id`, `POST /api/v1/opportunities/:id/approve`, `POST /api/v1/opportunities/:id/convert`, `GET /api/v1/projects`, `GET /api/v1/projects/:id`). 7. Configure the service to connect to PostgreSQL using credentials from the 'sigma1-infra-endpoints' ConfigMap.

### Subtasks
- [ ] Implement RMS Service - Core gRPC & REST Gateway (Grizz - Go/gRPC): Develop the core Rental Management System (RMS) service, focusing on Opportunity and Project management. This includes defining protobuf schemas, implementing gRPC services, and exposing them via a RESTful grpc-gateway.