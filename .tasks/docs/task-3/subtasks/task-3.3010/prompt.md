Implement subtask 3010: Implement CrewService gRPC handlers with Google Calendar API integration

## Objective
Implement the CrewService gRPC server for crew member management, project assignment, and availability checking integrated with Google Calendar API for scheduling.

## Steps
1. Create `/internal/service/crew_service.go` implementing the generated CrewServiceServer interface.
2. Implement CreateCrewMember, GetCrewMember, ListCrewMembers as standard CRUD.
3. Implement AssignCrewToProject:
   - Validate crew member and project exist.
   - Check for scheduling conflicts (overlapping assignments in `crew_assignments` table).
   - Insert crew_assignment record.
   - If crew member has a calendar_id, create a Google Calendar event for the assignment dates via Google Calendar API.
4. Implement UnassignCrewFromProject:
   - Update crew_assignment status.
   - Remove corresponding Google Calendar event if it exists.
5. Implement GetCrewAvailability:
   - Query crew_assignments for a given date range.
   - Optionally fetch Google Calendar free/busy info for the crew member.
   - Return merged availability windows.
6. Create `/internal/integrations/google_calendar.go`:
   - Use `google.golang.org/api/calendar/v3` client.
   - Authenticate with service account credentials (read from K8s secret).
   - Implement CreateEvent, DeleteEvent, GetFreeBusy methods.
   - Handle API errors gracefully with retries.
7. Create `/internal/repository/crew_repo.go`.
8. Register the service with the gRPC server.

## Validation
Unit tests with mocked Google Calendar client for all RPCs. Integration test: create crew member, assign to project, verify calendar event creation is called. Overlapping assignment is rejected. GetCrewAvailability returns correct windows. Google Calendar API failures are handled gracefully without crashing the service.