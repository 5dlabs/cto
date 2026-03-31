Implement subtask 2005: Implement RBAC middleware with hermes:read and hermes:trigger claim guards

## Objective
Create Hermes-specific RBAC middleware that extends the existing session middleware to check for hermes:read and hermes:trigger claims.

## Steps
1. In `src/modules/hermes/middleware.ts`, create `requireHermesClaim(claim: string)` function that returns an Elysia `beforeHandle` hook.
2. The middleware should:
   - Extract the current user session from the existing auth middleware (check project's auth pattern — likely `context.store.session` or `context.user`)
   - Check that the user's claims/roles include the required Hermes claim
   - Return 401 if no session, 403 if session exists but claim is missing
   - Return structured JSON error response: `{ error_code: 'HERMES_UNAUTHORIZED' | 'HERMES_FORBIDDEN', message: string }`
3. Define claim constants: `HERMES_CLAIMS = { READ: 'hermes:read', TRIGGER: 'hermes:trigger' }` in a shared location (can be in types.ts).
4. Export these claim constants for coordination with Task 10 (RBAC hardening).
5. Write the middleware to be composable — it should work as an Elysia guard/derive pattern.

## Validation
Unit test: `requireHermesClaim('hermes:trigger')` with a mock context containing the claim passes (calls next). Same middleware with a context missing the claim returns 403 with `HERMES_FORBIDDEN` error_code. No session returns 401 with `HERMES_UNAUTHORIZED`.