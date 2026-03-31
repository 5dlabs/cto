Implement subtask 3008: Implement ProjectService gRPC handlers

## Objective
Implement the ProjectService gRPC server with CRUD operations for projects, including status management and querying by opportunity linkage.

## Steps
1. Create `/internal/service/project_service.go` implementing the generated ProjectServiceServer interface.
2. Implement CreateProject: validate input, insert into `projects` table.
3. Implement GetProject: query by ID with related opportunity info, return 404 if not found.
4. Implement ListProjects: support pagination, status filtering, date range filtering.
5. Implement UpdateProject: field-mask-based updates, validate status transitions (PENDING→ACTIVE→COMPLETED, PENDING→CANCELLED, ACTIVE→CANCELLED).
6. Create `/internal/repository/project_repo.go` with data access methods.
7. Register the service with the gRPC server in main.go.

## Validation
Unit tests for each RPC method. Integration test: create project, update status through valid transitions, verify invalid transitions are rejected. List with filters returns correct subsets.