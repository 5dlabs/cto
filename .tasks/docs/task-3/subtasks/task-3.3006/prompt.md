Implement subtask 3006: Implement CrewService with scheduling logic

## Objective
Build the CrewService gRPC handler for crew member management and project scheduling, including availability tracking.

## Steps
1. Create /internal/service/crew_service.go implementing CrewServiceServer.
2. Implement CreateMember: persist crew member with role and initial availability.
3. Implement GetMember, ListMembers: support filtering by role, availability; pagination.
4. Implement UpdateMember: partial updates for role, contact info, availability.
5. Implement GetSchedule: accept crew_member_id and date range, query project_crew join table to return all project assignments within the range, along with project details (name, dates).
6. Implement AssignToProject: validate crew member exists and is available for the project date range (no overlapping assignments), insert into project_crew join table.
7. Add availability conflict detection: query existing assignments for overlapping date ranges before allowing new assignment.
8. Register service on gRPC server and grpc-gateway mux.

## Validation
AssignToProject succeeds when crew member has no conflicts and fails with ALREADY_EXISTS or FAILED_PRECONDITION when date range overlaps; GetSchedule returns correct project assignments for a date range; ListMembers filters by availability correctly.