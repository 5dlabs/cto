Implement subtask 3004: Implement ProjectService with quote-to-project conversion and Google Calendar integration

## Objective
Build the ProjectService including atomic quote-to-project conversion (marking opportunity as converted and creating project in a single transaction) and Google Calendar API integration for scheduling.

## Steps
1. Create `internal/project/` package with repository, service, and handler layers.
2. Implement `repository.go` with PostgreSQL queries: CreateProject, GetProjectByID, ListProjects, UpdateProject.
3. Implement the ConvertToProject flow as a database transaction: (a) verify opportunity status is 'accepted', (b) update opportunity status to 'converted', (c) insert new project linked to opportunity_id, (d) copy relevant data (line items become project items). Rollback entire transaction on any failure.
4. Create `internal/calendar/` package wrapping Google Calendar API client.
5. Implement calendar.CreateEvent, calendar.UpdateEvent, calendar.DeleteEvent methods.
6. On project creation/update with dates, create/update a Google Calendar event and store the calendar_event_id on the project record.
7. Handle calendar API failures gracefully: log the error, mark calendar_sync_pending flag, do not fail the project creation.
8. Wire up gRPC handlers and register service.

## Validation
Integration test: create an opportunity, accept it, convert to project — verify opportunity is 'converted' and project exists atomically. Verify converting a non-accepted opportunity fails. Mock Google Calendar API client: verify event creation is called with correct dates. Verify calendar failure does not roll back project creation.