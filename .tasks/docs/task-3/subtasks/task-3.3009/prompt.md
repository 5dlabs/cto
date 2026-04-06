Implement subtask 3009: Integrate Google Calendar API for crew and project scheduling

## Objective
Implement the Google Calendar API client and wire it into CrewService.ScheduleCrew and ProjectService for calendar event creation and availability checking.

## Steps
1. Create `internal/calendar/google_calendar.go` with a CalendarClient interface and Google Calendar API v3 implementation. 2. Implement authentication using a service account (credentials from environment/secret). 3. Implement CreateEvent(calendarId, summary, startTime, endTime, attendees) → eventId. 4. Implement ListEvents(calendarId, timeMin, timeMax) for availability checks. 5. Implement UpdateEvent and DeleteEvent for schedule changes. 6. Wire into CrewService.ScheduleCrew: when scheduling a crew member, create a calendar event on their calendar_id and store the event ID. 7. Wire into CrewService.CheckAvailability: query Google Calendar for free/busy information alongside DB assignments. 8. Wire into ProjectService.CreateProject/UpdateProject: optionally create/update a project calendar event. 9. Handle Google API errors gracefully (rate limits, auth failures) with retries and meaningful gRPC error responses. 10. Make Calendar integration optional via feature flag so the service works without Calendar credentials in dev/test.

## Validation
Unit tests with mocked CalendarClient verify correct API calls are made; integration test with Google Calendar sandbox: create event, query free/busy, update event; ScheduleCrew creates a calendar event and stores the event ID; feature flag disables Calendar calls gracefully.