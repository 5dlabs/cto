Implement subtask 3004: Implement OpportunityService and ProjectService gRPC handlers

## Objective
Build the gRPC service implementations for OpportunityService and ProjectService with full business logic, including the opportunity-to-project conversion workflow.

## Steps
1. Create /internal/service/opportunity_service.go implementing the generated OpportunityServiceServer interface.
2. Implement CreateOpportunity: validate input, persist via OpportunityRepo, return created opportunity.
3. Implement GetOpportunity: fetch by ID, return NOT_FOUND if missing.
4. Implement ListOpportunities: support filtering by status, customer_id; pagination with page_token/page_size.
5. Implement UpdateOpportunity: field-mask based partial update, optimistic concurrency check.
6. Implement ConvertToProject: within a DB transaction, create a new Project from the opportunity data, update opportunity status to WON, return the new project. This is the core quote-to-project workflow entry point.
7. Create /internal/service/project_service.go implementing ProjectServiceServer.
8. Implement CreateProject, GetProject, ListProjects (filter by status, date range), UpdateProject.
9. Implement AssignCrew: link crew member IDs to project via project_crew join table.
10. Implement AssignEquipment: link equipment IDs to project via project_equipment join table, validate equipment availability status.
11. Register both services on the gRPC server and grpc-gateway mux in main.go.
12. Add input validation using a validation library or custom validators for required fields.

## Validation
Unit tests for each RPC method with mocked repositories; ConvertToProject correctly creates project and updates opportunity in a single transaction; ListOpportunities pagination returns correct pages; gRPC and REST endpoints both return expected responses via grpc-gateway.