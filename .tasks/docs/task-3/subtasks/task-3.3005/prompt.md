Implement subtask 3005: Implement database repository layer with pgx

## Objective
Build the Go repository layer using pgx/v5 for all RMS entities with org_id-scoped queries, providing CRUD operations consumed by gRPC service implementations.

## Steps
1. Create `internal/repo/` package with one file per entity: `opportunity_repo.go`, `project_repo.go`, `inventory_repo.go`, `crew_repo.go`, `delivery_repo.go`.
2. Define repository interfaces in `internal/repo/interfaces.go` for each entity (e.g., `OpportunityRepo` with Create, Get, Update, List, UpdateStatus methods).
3. `opportunity_repo.go`: Implement `Create(ctx, orgID, opp)`, `GetByID(ctx, orgID, id)`, `Update(ctx, orgID, id, opp)`, `List(ctx, orgID, filters, pagination)`, `UpdateStatus(ctx, orgID, id, newStatus)`. All queries filter by `org_id = $orgID`. Use `pgx.CollectRows` for list queries.
4. `project_repo.go`: Implement `Create`, `GetByID`, `Update`, `List`, `CreateFromOpportunity(ctx, orgID, oppID)` which inserts project and copies line items in a transaction.
5. `inventory_repo.go`: Implement `GetByID`, `GetByBarcode(ctx, orgID, barcode)`, `GetStockLevel(ctx, orgID, itemID)`, `RecordTransaction(ctx, orgID, txn)` which updates quantity_available atomically in a transaction, `CheckAvailability(ctx, orgID, itemIDs, dateStart, dateEnd)` which queries overlapping transactions.
6. `crew_repo.go`: Implement `List(ctx, orgID)`, `CreateAssignment(ctx, orgID, assignment)`, `GetConflicts(ctx, orgID, crewMemberID, dateStart, dateEnd)` which queries overlapping crew_assignments.
7. `delivery_repo.go`: Implement `Create`, `UpdateStatus`, `List`, `SaveRoute`.
8. Use `pgx.Pool` injected into each repo struct. Use squirrel or raw SQL with parameterized queries.
9. All methods return domain structs (not protobuf types) defined in `internal/domain/` package.

## Validation
Integration tests with testcontainers-go PostgreSQL: for each repo, test Create→Get roundtrip, verify org_id filtering (insert with org_id A, query with org_id B returns empty), test List with pagination. For inventory repo, test RecordTransaction atomically updates quantity_available. For crew repo, test GetConflicts returns overlapping assignments.