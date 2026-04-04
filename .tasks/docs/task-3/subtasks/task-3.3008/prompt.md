Implement subtask 3008: Implement Crew scheduling service with conflict detection

## Objective
Build the CrewService gRPC implementation with crew listing, assignment, scheduling with overlap conflict detection, and availability queries.

## Steps
1. Create `internal/service/crew_svc.go` implementing `CrewServiceServer`.
2. Implement `ListCrew` RPC: delegate to `crewRepo.List(ctx, orgID)` with optional skill/role filters.
3. Implement `AssignCrew` RPC:
   - Accept crew_member_id, project_id, date_start, date_end, role.
   - Call `crewRepo.GetConflicts(ctx, orgID, crewMemberID, dateStart, dateEnd)` to check for overlapping assignments.
   - If conflicts exist, return error with details of conflicting assignments (project_id, dates).
   - If no conflicts, call `crewRepo.CreateAssignment(ctx, orgID, assignment)`.
4. Implement `ScheduleCrew` RPC: bulk assign multiple crew members to a project, checking conflicts for each. Return summary of successful assignments and any conflicts.
5. Implement `GetCrewAvailability` RPC:
   - For a given crew_member_id and date range, query existing assignments and return available time slots.
   - Return list of `AvailabilitySlot` (date_start, date_end, is_available).
6. Register service in gRPC server.

## Validation
Integration tests: 1) Create crew member, assign to project A for Jan 1-5, attempt assign to project B for Jan 3-7 → verify conflict error with project A details. 2) Assign to non-overlapping period Jan 6-10 → verify success. 3) GetCrewAvailability for Jan 1-15 → verify slot Jan 1-5 shows unavailable, Jan 6-10 unavailable, Jan 11-15 available. 4) ScheduleCrew bulk: 3 crew members, 1 has conflict → verify 2 succeed and 1 returns conflict.