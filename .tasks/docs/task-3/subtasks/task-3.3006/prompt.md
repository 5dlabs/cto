Implement subtask 3006: Implement CrewService with scheduling and calendar sync

## Objective
Build the CrewService for managing crew members, assigning them to projects with date ranges, detecting scheduling conflicts, and syncing assignments to Google Calendar.

## Steps
1. Create `internal/crew/` package with repository, service, and handler layers.
2. Implement `repository.go`: CreateCrewMember, GetCrewMemberByID, ListCrewMembers, CreateAssignment, DeleteAssignment, ListAssignmentsByProject, ListAssignmentsByCrewMember, FindConflictingAssignments(crew_member_id, start_date, end_date).
3. Implement assignment logic: before creating an assignment, query for overlapping assignments for the same crew member. If conflicts found, return FailedPrecondition with details of the conflicting assignment.
4. Integrate with the `internal/calendar/` package (from task 3004): when creating an assignment, create a Google Calendar event for the crew member's schedule. Store calendar_event_id on the crew_assignment record.
5. Implement availability checking: given a date range, return which crew members have no conflicting assignments.
6. Wire up gRPC handlers.

## Validation
Unit tests for conflict detection logic with overlapping date ranges. Integration tests: create crew members, assign to a project for date range, attempt to assign same crew member to overlapping project — verify conflict error. Verify availability endpoint correctly filters out booked crew. Mock calendar client: verify events are created for assignments.