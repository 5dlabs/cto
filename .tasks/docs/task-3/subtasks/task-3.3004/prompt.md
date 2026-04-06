Implement subtask 3004: Implement OpportunityService gRPC handlers with PostgreSQL integration

## Objective
Implement the OpportunityService server including all CRUD operations and the ConvertToProject workflow, backed by PostgreSQL queries.

## Steps
1. Create `internal/service/opportunity/service.go` implementing the generated OpportunityServiceServer interface. 2. Implement CreateOpportunity: validate input, insert into `rms.opportunities`, return created record. 3. Implement GetOpportunity: query by ID with tenant_id filter, return 404 if not found. 4. Implement ListOpportunities: support pagination (page_size, page_token), filtering by status, and sorting. 5. Implement UpdateOpportunity: partial update using field masks or explicit fields, optimistic concurrency via updated_at. 6. Implement ConvertToProject: transactionally update opportunity status to WON and insert a new project record linked to the opportunity; return the new project. 7. Create `internal/repository/opportunity_repo.go` with a clean repository interface for DB operations using sqlx or pgx. 8. Register the service with the gRPC server in main.go.

## Validation
Unit tests with mocked repository pass for all RPCs; integration test against real PostgreSQL: create, read, update, list, and convert-to-project all succeed; ConvertToProject is atomic (opportunity status + project creation in one transaction); gRPC error codes are correct for not-found and validation failures.