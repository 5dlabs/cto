Implement subtask 3005: Implement OpportunityService and ProjectService with quote-to-project workflow

## Objective
Implement the gRPC handlers for OpportunityService and ProjectService, including the critical ConvertToProject business logic that transitions an accepted quote into an active project.

## Steps
1. Implement OpportunityService handlers in /internal/opportunity/service.go: CreateOpportunity (validate inputs, insert into DB), GetOpportunity, ListOpportunities (with pagination and status filtering), UpdateOpportunity (status transitions: draft→quoted→accepted/rejected). 2. Implement ConvertToProject RPC: validate opportunity status is 'accepted', begin a database transaction, create a new project record linked to the opportunity, update opportunity status to 'converted', commit transaction, return the new project. 3. Implement ProjectService handlers in /internal/project/service.go: GetProject (join with opportunity data), ListProjects (with status and date filtering), UpdateProject (status transitions with validation), GetProjectTimeline (aggregate assigned crew, inventory, deliveries). 4. Use repository pattern for database access. 5. Add input validation on all requests. 6. Return proper gRPC status codes (NotFound, InvalidArgument, FailedPrecondition for invalid state transitions).

## Validation
Unit tests cover all OpportunityService and ProjectService handlers with mocked repositories; ConvertToProject correctly creates a project and updates opportunity status atomically; invalid state transitions return FailedPrecondition; integration test runs ConvertToProject against a real database and verifies both tables are updated; >80% code coverage.