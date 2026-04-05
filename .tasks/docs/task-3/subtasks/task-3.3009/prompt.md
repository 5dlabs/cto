Implement subtask 3009: Implement CrewService gRPC handlers with scheduling conflict detection

## Objective
Implement the CrewService gRPC server including crew listing, assignment creation with overlap conflict detection, and bulk scheduling.

## Steps
1. Create `internal/service/crew.go` implementing CrewServiceServer.
2. Implement ListCrew:
   - Paginated list of crew members
   - Support optional filter by role
3. Implement AssignCrew:
   - Accept project_id, crew_member_id, role, start_time, end_time
   - Validate project exists and is in active status (confirmed or in_progress)
   - **Conflict detection**: Query crew_assignments for the given crew_member_id where time ranges overlap: `existing.start_time < new.end_time AND existing.end_time > new.start_time`
   - If overlapping assignment found, return AlreadyExists error with details about the conflicting assignment (project_id, times)
   - If no conflict, insert assignment
   - Return created assignment
4. Implement ScheduleCrew:
   - Accept project_id and list of assignment requests
   - Validate all assignments for conflicts (batch check)
   - Insert all in a single database transaction — if any conflict, roll back all and return error indicating which assignments conflicted
5. Use SELECT FOR UPDATE or advisory locks to prevent race conditions in concurrent assignment requests for the same crew member.
6. Register service in gRPC server.

## Validation
Integration test: assign crew member A to project 1 (10am-2pm), then assign crew member A to project 2 (1pm-5pm) — verify error returned with conflict details. Assign crew member A to project 2 (3pm-6pm after first is 10am-2pm) — verify success (no overlap). Test ScheduleCrew with batch of 3 where 1 conflicts — verify all rolled back. Test concurrent assignment requests for same crew member don't cause double-booking.