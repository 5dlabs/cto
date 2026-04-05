Implement subtask 3009: Integrate Google Calendar API for crew and project scheduling

## Objective
Add Google Calendar API integration so that crew assignments and project schedules are synced bidirectionally with Google Calendar.

## Steps
1. Create /internal/calendar/google.go with Google Calendar API client setup using the official google.golang.org/api/calendar/v3 package.
2. Implement OAuth2 service account authentication for the Calendar API; read credentials from a mounted secret (referenced via sigma1-infra-endpoints or a dedicated secret).
3. Implement CreateCalendarEvent: given a project name, crew member email, start/end times, create a calendar event and return the event ID.
4. Implement UpdateCalendarEvent: update an existing event when project dates change.
5. Implement DeleteCalendarEvent: remove event when crew is unassigned from a project.
6. Hook into CrewService.AssignToProject: after successful DB assignment, create a calendar event and store the event_id on the crew-project association.
7. Hook into ProjectService.UpdateProject: if dates change, update all associated calendar events.
8. Handle API errors gracefully: if Calendar API fails, log the error but don't fail the core operation; mark calendar sync as pending for retry.
9. Define a CalendarService interface to allow mocking in tests.

## Validation
Unit tests with mocked Google Calendar client verify event creation, update, and deletion are called with correct parameters; integration test (if credentials available) creates and deletes a test event; core operations succeed even when Calendar API returns errors.