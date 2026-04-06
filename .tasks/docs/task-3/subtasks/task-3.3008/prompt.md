Implement subtask 3008: Implement CrewService gRPC handlers with PostgreSQL integration

## Objective
Implement the CrewService server with CRUD and availability checking, backed by PostgreSQL. This subtask covers only the database-backed operations, not the Google Calendar integration.

## Steps
1. Create `internal/service/crew/service.go` implementing CrewServiceServer. 2. Implement CreateCrewMember: validate input, insert into `rms.crew_members`. 3. Implement GetCrewMember: query by ID. 4. Implement ListCrewMembers: pagination, filtering by role, skills, availability_status. 5. Implement UpdateCrewMember: partial updates for details, skills, availability. 6. Implement CheckAvailability: query crew member's current project assignments from `rms.project_crew`, return availability windows (database-only, Calendar integration comes separately). 7. Stub ScheduleCrew to accept the request and persist a placeholder — the Google Calendar call will be wired in the next subtask. 8. Create `internal/repository/crew_repo.go`. 9. Register service.

## Validation
Unit tests cover all RPCs; integration tests: create crew members, check availability against project assignments, verify skill filtering works; ScheduleCrew persists the assignment in the DB even without Calendar integration.