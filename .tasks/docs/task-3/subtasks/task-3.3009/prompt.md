Implement subtask 3009: Implement Google Calendar integration for crew/project sync

## Objective
Build Google Calendar API integration to sync project events and crew assignments to Google Calendar using OAuth2 service account credentials.

## Steps
1. Create `internal/gcal/client.go` package.
2. Initialize Google Calendar API client using service account JSON key from K8s Secret (env var `GOOGLE_CALENDAR_SA_KEY` or file path `GOOGLE_APPLICATION_CREDENTIALS`).
3. Implement `SyncProjectEvent(ctx, project Project) (eventID string, error)`:
   - Create/update a Google Calendar event with project title, date range, description, and assigned crew as attendees.
   - Use calendar ID from config (env var `GOOGLE_CALENDAR_ID`).
   - Store returned event ID on the project record for future updates.
4. Implement `SyncCrewAssignment(ctx, assignment CrewAssignment, crewEmail string) error`:
   - Create a calendar event for the crew member's assignment period.
   - Include project details in event description.
5. Implement `DeleteEvent(ctx, eventID string) error` for cleanup on project/assignment deletion.
6. Add an interface `CalendarSyncer` in `internal/gcal/interface.go` to allow mocking in tests.
7. Integrate into OpportunityService's `ConvertOpportunity` (create calendar event for new project) and CrewService's `AssignCrew` (create event for assignment).
8. Handle API errors gracefully: log and continue if Calendar API is unavailable (non-blocking sync).

## Validation
Unit tests with mock CalendarSyncer: verify ConvertOpportunity calls SyncProjectEvent with correct project data, verify AssignCrew calls SyncCrewAssignment with correct crew email and dates. Test error handling: mock Calendar API failure, verify assignment still succeeds and error is logged. Integration test (optional, requires test calendar): create event, verify it appears, update it, delete it.