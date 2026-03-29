Implement subtask 3004: Implement Opportunity and Project gRPC service logic

## Objective
Implement the server-side logic for `OpportunityService` and `ProjectService`, including quote-to-project conversion.

## Steps
1. Implement gRPC handlers for `CreateOpportunity`, `GetOpportunity`, `UpdateOpportunity`, `ConvertOpportunityToProject`.2. Implement gRPC handlers for `CreateProject`, `GetProject`, `UpdateProject`.

## Validation
1. Use `grpcurl` to test `CreateOpportunity`, `ConvertOpportunityToProject`, and `GetProject` endpoints.2. Verify data consistency in PostgreSQL after these operations.