Implement subtask 3007: Implement CrewService with scheduling logic

## Objective
Implement gRPC handlers for CrewService including crew member CRUD, availability checking, and crew-to-project scheduling with conflict detection.

## Steps
1. Implement CrewService handlers in /internal/crew/service.go: CreateCrewMember, GetCrewMember, ListCrewMembers (filter by role, availability_status). 2. Implement GetAvailability RPC: given a crew_member_id and date range, query crew_assignments to find conflicts, return available time slots. 3. Implement ScheduleCrewMember RPC: accept crew_member_id, project_id, start_date, end_date, role_on_project. Check for overlapping assignments in the date range. If no conflicts, insert crew_assignment record. If conflict exists, return FailedPrecondition with details of the conflicting assignment. 4. Add UpdateCrewMember for profile/rate changes. 5. Use proper gRPC error codes throughout.

## Validation
Unit tests cover availability checking with overlapping and non-overlapping assignments; ScheduleCrewMember rejects double-booking; integration test assigns crew to project and verifies availability is reduced; >80% coverage.