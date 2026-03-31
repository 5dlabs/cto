Implement subtask 3007: Implement OpportunityService gRPC handlers with quote-to-project conversion

## Objective
Implement the OpportunityService gRPC server including CRUD operations and the ConvertOpportunityToProject RPC that transitions an opportunity into a project (the core quote-to-project workflow).

## Steps
1. Create `/internal/service/opportunity_service.go` implementing the generated OpportunityServiceServer interface.
2. Implement CreateOpportunity: validate input, insert into `opportunities` table, return created record.
3. Implement GetOpportunity: query by ID, return 404 if not found.
4. Implement ListOpportunities: support pagination (page_size, page_token), optional status filtering.
5. Implement UpdateOpportunity: field-mask-based updates, validate status transitions.
6. Implement ConvertOpportunityToProject:
   - Validate opportunity exists and is in WON status.
   - Begin database transaction.
   - Create a new project record linked to the opportunity.
   - Update opportunity status to indicate conversion.
   - Commit transaction.
   - Return the newly created project.
7. Create `/internal/repository/opportunity_repo.go` with data access methods using pgxpool.
8. Register the service with the gRPC server in main.go.

## Validation
Unit tests for each RPC method using a mock repository. Integration test: create an opportunity, update it to WON status, convert to project — verify project is created and opportunity is updated. ConvertOpportunityToProject on a non-WON opportunity returns an appropriate error.