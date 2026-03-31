Implement subtask 5005: Implement admin API endpoint for migration trigger with RBAC

## Objective
Create `POST /api/hermes/admin/migrate-artifacts` Elysia route that starts a migration job asynchronously, requires `hermes:admin` RBAC claim, and returns a 202 with a job ID for tracking.

## Steps
Step-by-step:
1. Create `src/modules/hermes/routes/admin-migration.ts` with an Elysia route group under `/api/hermes/admin`.
2. Add RBAC middleware that checks for `hermes:admin` claim in the JWT/auth token. Return 403 with `{ error: 'Forbidden', message: 'Requires hermes:admin claim' }` if missing.
3. `POST /api/hermes/admin/migrate-artifacts` handler:
   a. Generate a unique migration job ID (UUID).
   b. Kick off `migrator.migrate()` asynchronously (do not await in the request handler).
   c. Return 202 with `{ jobId: string, status: 'started', message: 'Migration job started' }`.
4. Optionally accept request body: `{ batchSize?: number, dryRun?: boolean }`.
5. Prevent concurrent migrations: if a migration is already running, return 409 with `{ error: 'Conflict', message: 'Migration already in progress' }`.
6. Register the route in the main Elysia app's plugin chain.

## Validation
Test 403: Send POST to `/api/hermes/admin/migrate-artifacts` without auth token — verify 403 response. Test 403 with wrong claim: send with a valid token lacking `hermes:admin` — verify 403. Test 202: send with valid `hermes:admin` token — verify 202 response with `jobId` field. Test 409: start a migration, immediately send another POST — verify 409 response. Test dry-run: send with `{ dryRun: true }` — verify migration runs in scan-only mode.