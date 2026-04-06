Implement subtask 3005: Implement ProjectService gRPC handlers with PostgreSQL integration

## Objective
Implement the ProjectService server with CRUD operations and crew assignment, backed by PostgreSQL.

## Steps
1. Create `internal/service/project/service.go` implementing ProjectServiceServer. 2. Implement CreateProject: validate input, insert into `rms.projects`, return created record. 3. Implement GetProject: query by ID, include related opportunity data if needed. 4. Implement ListProjects: support pagination, filtering by status, date range. 5. Implement UpdateProject: partial updates, status transitions with validation (e.g., cannot go from COMPLETED back to PLANNING). 6. Implement AssignCrew: insert into `rms.project_crew` join table, validate crew member exists and is available. 7. Create `internal/repository/project_repo.go` with repository pattern. 8. Register service with gRPC server.

## Validation
Unit tests with mocked repository cover all RPCs and status transition validation; integration tests: create project, assign crew, verify join table entries; invalid status transitions return appropriate gRPC error codes.