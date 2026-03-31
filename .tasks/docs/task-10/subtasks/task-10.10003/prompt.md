Implement subtask 10003: Finalize application-level RBAC claim taxonomy and create database migration

## Objective
Define the RBAC claim taxonomy (hermes:read, hermes:trigger, hermes:admin, hermes:delete), coordinate with Task 2's session model, and create a database migration to add claims to existing admin users.

## Steps
1. Define the final claim taxonomy in a shared constants file (e.g., `src/modules/hermes/constants.ts`):
   - `hermes:read` — view deliberations and artifacts
   - `hermes:trigger` — create new deliberations
   - `hermes:admin` — trigger migrations, access admin endpoints
   - `hermes:delete` — delete deliberations and artifacts (optional, for future use)
2. Create a database migration file (e.g., `migrations/XXXXXX_add_hermes_claims.sql`) that:
   - Adds a `claims` JSONB column to the user/session table if not already present (coordinate with Task 2's schema).
   - Updates existing admin users to have `["hermes:read", "hermes:trigger", "hermes:admin"]` claims.
3. Ensure the session model exposes claims for middleware consumption.
4. Document claim assignment workflow for new users.

## Validation
Run the migration against a test database and verify: (1) The claims column exists. (2) Admin users have the expected claims array. (3) A SELECT query for a specific user returns the correct claims. Verify the constants file exports all four claim values.