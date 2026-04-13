<identity>
You are grizz working on subtask 3009 of task 3.
</identity>

<context>
<scope>
On ProjectService UpdateProject when status transitions to confirmed, create a Google Calendar event via the Calendar v3 API and store the returned event ID in the project row.
</scope>
</context>

<implementation_plan>
Create internal/calendar/client.go. Initialize a *calendar.Service using golang.org/x/oauth2/google and google.golang.org/api/calendar/v3. Read GOOGLE_CALENDAR_CLIENT_ID and GOOGLE_CALENDAR_CLIENT_SECRET from environment (injected via sigma1-google-secret). Use a service account or stored refresh token approach (confirm via decision point). In the UpdateProject handler, detect when new status == 'confirmed' and old status != 'confirmed'. Call calendarClient.Events.Insert(calendarId, &calendar.Event{Summary: project.venue_address, Start: {DateTime: event_date_start}, End: {DateTime: event_date_end}}).Do(). On success, UPDATE projects SET calendar_event_id = returnedEventID WHERE id = project.id. On Calendar API error, log the error and do not fail the gRPC call (best-effort integration).
</implementation_plan>

<validation>
In integration test with a mocked Calendar HTTP server (httptest), UpdateProject with status=confirmed triggers an Events.Insert call and the project row shows a non-empty calendar_event_id. Repeat call with same status does not duplicate the event. Calendar API error does not cause UpdateProject to return an error.
</validation>