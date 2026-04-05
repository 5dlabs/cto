Implement subtask 3008: Integrate Google Calendar API for project scheduling

## Objective
Implement Google Calendar integration to sync project schedules, crew assignments, and delivery events to a shared calendar.

## Steps
1. Add Google Calendar API Go client dependency. 2. Implement a calendar package in /internal/calendar/ that authenticates using a service account key (stored as a Kubernetes secret). 3. Implement CreateCalendarEvent: given a project with dates, crew, and equipment, create a Google Calendar event with relevant details in the description. 4. Implement UpdateCalendarEvent and DeleteCalendarEvent for schedule changes. 5. Hook into ProjectService (on project confirmation) and DeliveryService (on delivery scheduling) to automatically create/update calendar events. 6. Handle API errors gracefully — calendar sync failures should log warnings but not block the primary operation.

## Validation
Calendar events are created when a project is confirmed (verified via mock or test calendar); events update when project schedule changes; calendar API failures are logged but do not cause RPC errors; service account authentication works correctly.