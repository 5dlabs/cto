Implement subtask 3008: Integrate Google Calendar API for crew scheduling events

## Objective
Add Google Calendar integration to CrewService so that when crew members are scheduled to projects, calendar events are created/updated/deleted on their linked Google Calendar.

## Steps
1. Create /internal/calendar/google.go with a Google Calendar API client. 2. Set up authentication (service account or OAuth2 depending on decision point resolution) using credentials from Kubernetes secrets. 3. Implement CreateEvent: given crew_member google_calendar_id, project name, start/end dates, create a Google Calendar event. Return the event ID and store it in the crew_assignment record. 4. Implement UpdateEvent: when a crew assignment date range changes, update the corresponding calendar event. 5. Implement DeleteEvent: when a crew assignment is removed, delete the calendar event. 6. Call calendar functions from ScheduleCrewMember RPC after successful DB write. Use a best-effort pattern: if Calendar API fails, log the error but don't fail the scheduling operation (the assignment is still valid). 7. Add a reconciliation function that can be called to sync DB assignments with calendar events.

## Validation
Unit tests with mocked Google Calendar client verify events are created with correct data; integration test with Google Calendar sandbox verifies event creation and deletion; scheduling still succeeds if Calendar API returns an error (best-effort).