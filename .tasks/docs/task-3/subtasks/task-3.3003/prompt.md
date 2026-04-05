Implement subtask 3003: Implement OpportunityService with quote workflow

## Objective
Build the OpportunityService gRPC server implementation including CRUD operations for opportunities/quotes, status transitions (draft → sent → accepted → converted), and line item management.

## Steps
1. Create `internal/opportunity/` package with repository, service, and handler layers.
2. Implement `repository.go` with PostgreSQL queries: CreateOpportunity, GetOpportunityByID, ListOpportunities (with filtering/pagination), UpdateOpportunityStatus, AddLineItem, RemoveLineItem.
3. Implement `service.go` with business logic: validate status transitions (draft→sent→accepted→rejected, accepted→converted), calculate total amount from line items, enforce required fields per status.
4. Implement `handler.go` registering the gRPC server interface.
5. Wire up in main.go server registration.
6. Ensure all mutations include audit fields (created_by, updated_by, timestamps).
7. Add input validation using a validation library (e.g., go-playground/validator).
8. Return proper gRPC status codes (NotFound, InvalidArgument, FailedPrecondition for invalid transitions).

## Validation
Unit tests for status transition validation logic. Integration tests calling gRPC endpoints: create an opportunity, add line items, verify total calculation, transition through statuses. Verify invalid transitions return FailedPrecondition. REST endpoints via grpc-gateway return equivalent JSON responses.