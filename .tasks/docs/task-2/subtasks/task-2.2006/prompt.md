Implement subtask 2006: Implement REST endpoints and register Elysia plugin with feature flag

## Objective
Implement all four Hermes REST endpoints as Elysia route handlers, wire them to the service layer with RBAC guards, register as a plugin, and implement the HERMES_ENABLED feature flag.

## Steps
1. In `src/modules/hermes/routes.ts`, implement four route handlers using Elysia:
   - `POST /api/hermes/deliberations` — guard: `hermes:trigger`. Parse body, call `service.triggerDeliberation()`, return 201 with deliberation object.
   - `GET /api/hermes/deliberations/:id` — guard: `hermes:read`. Parse UUID param, call `service.getDeliberation()`, return 200 or 404.
   - `GET /api/hermes/deliberations` — guard: `hermes:read`. Parse pagination query params, call `service.listDeliberations()`, return 200 with paginated response.
   - `GET /api/hermes/deliberations/:id/artifacts` — guard: `hermes:read`. Call `service.getDeliberationArtifacts()`, return 200 with artifacts array.
2. Apply Elysia validation schemas (using `t.Object()` from Elysia's typebox) for request bodies and query params.
3. In `src/modules/hermes/index.ts`, create `hermesPlugin` as an Elysia instance/plugin:
   - Check `process.env.HERMES_ENABLED` — if not `'true'`, return an empty plugin (no routes registered).
   - Instantiate `HermesRepository`, `HermesArtifactWriter`, and `HermesService` with dependency injection.
   - Mount all routes.
4. In the main app entry point, register: `app.use(hermesPlugin)`.
5. Ensure Hermes routes do NOT conflict with any existing legacy pipeline routes.
6. All endpoints return structured JSON error responses with `error_code` on failure.

## Validation
Integration test: `POST /api/hermes/deliberations` with valid session and `hermes:trigger` claim returns 201 with deliberation ID. Same without claim returns 403. `GET /api/hermes/deliberations/:id` returns correct record. `GET /api/hermes/deliberations` returns paginated list. When `HERMES_ENABLED=false`, all Hermes routes return 404.