## Acceptance Criteria

- [ ] 1. Unit tests: `HermesService.triggerDeliberation()` creates a deliberation record in PostgreSQL with status `pending` — verified by direct DB query returning the UUID.
- [ ] 2. Integration test: `POST /api/hermes/deliberations` with valid session and `hermes:trigger` claim returns 201 with a deliberation ID; same request without `hermes:trigger` claim returns 403.
- [ ] 3. Integration test: `GET /api/hermes/deliberations/:id` returns the correct deliberation record with status field matching the DB state.
- [ ] 4. OpenAPI spec: `GET /api/swagger/json` includes all four Hermes endpoints with correct request/response schemas.
- [ ] 5. Feature flag: When `HERMES_ENABLED=false`, `GET /api/hermes/deliberations` returns 404 (routes not registered).
- [ ] 6. Database migration: Running migrations on a clean database creates `deliberations` and `hermes_artifacts` tables without errors; running on an existing database with legacy data does not alter existing tables.

## Verification Notes

- [ ] Confirm dependencies are satisfied before implementation.
- [ ] Update tests, docs, and configuration touched by this task.
- [ ] Validate the final behavior against the task objective.