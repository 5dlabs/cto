Implement subtask 3005: Implement database access layer with pgxpool connection management

## Objective
Create the database access layer using pgxpool for connection pooling, including repository interfaces and implementations for all 7 tables with standard CRUD operations and query helpers.

## Steps
1. Add `github.com/jackc/pgx/v5` and `github.com/jackc/pgx/v5/pgxpool` to go.mod.
2. Create `internal/db/pool.go`: NewPool function that reads DATABASE_URL from env (via sigma1-infra-endpoints ConfigMap), configures pool (max conns, min conns, health check period, max conn lifetime).
3. Create repository interfaces in `internal/db/` for each domain:
   - `opportunity_repo.go`: OpportunityRepo interface with Create, GetByID, Update, List (with pagination + status filter), CreateLineItem, ListLineItems
   - `project_repo.go`: ProjectRepo interface with Create, GetByID, Update, GetByOpportunityID
   - `inventory_repo.go`: InventoryRepo interface with RecordTransaction, GetStockLevel (aggregates checkouts - checkins), LookupByBarcode
   - `crew_repo.go`: CrewRepo interface with ListMembers, CreateAssignment, ListAssignmentsByProject, FindOverlappingAssignments (for conflict detection)
   - `delivery_repo.go`: DeliveryRepo interface with Create, GetByID, UpdateStatus, ListByProject
4. Implement each repository using pgx query builder with named parameters and row scanning.
5. Use pgx.Batch for bulk operations where applicable.
6. All repository methods accept `context.Context` as first parameter for cancellation and tracing.
7. Create `internal/db/db.go` that initializes all repositories and exposes them via a `Store` struct.

## Validation
Write unit tests with pgxmock for each repository method verifying correct SQL generation. Write integration test that connects to a test PostgreSQL instance, runs migrations, and performs CRUD on each table. Verify connection pool metrics (active connections, idle connections) are accessible.