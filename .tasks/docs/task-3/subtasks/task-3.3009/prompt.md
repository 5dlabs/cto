Implement subtask 3009: Integrate Google Calendar API for project scheduling

## Objective
Implement Google Calendar API integration within ProjectService to create, update, and delete calendar events when projects are created or modified.

## Steps
1. Add google.golang.org/api/calendar/v3 and golang.org/x/oauth2 dependencies.
2. Create internal/calendar/google_calendar.go with a CalendarClient interface and Google implementation:
   - CreateEvent(project) → creates a Google Calendar event with project name, dates, crew info, returns event ID
   - UpdateEvent(eventID, project) → updates existing event
   - DeleteEvent(eventID) → removes event
3. Read Google Calendar credentials (service account JSON or OAuth tokens) from secrets mounted via sigma1-infra-endpoints.
4. Configure the target calendar ID from environment/ConfigMap.
5. Integrate into ProjectService:
   - On CreateProject, call CalendarClient.CreateEvent and store returned event ID in project record
   - On UpdateProject (date/crew changes), call CalendarClient.UpdateEvent
   - On DeleteProject, call CalendarClient.DeleteEvent
6. Handle API errors gracefully: log and continue if calendar sync fails (don't block project CRUD).
7. Implement SyncCalendar RPC for manual re-sync of a project's calendar event.
8. Create a mock CalendarClient for testing.

## Validation
Unit tests with mock CalendarClient verify that CreateProject calls CreateEvent and stores event ID; UpdateProject calls UpdateEvent; DeleteProject calls DeleteEvent; calendar API failures are logged but don't cause project operations to fail; SyncCalendar RPC triggers event update; integration test against Google Calendar sandbox (if available) creates and retrieves an event.