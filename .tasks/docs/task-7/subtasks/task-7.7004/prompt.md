Implement subtask 7004: Implement API test suite for authentication and authorization

## Objective
Create API-level spec files that validate all authentication and authorization scenarios: unauthenticated requests, insufficient claims, and correct claims.

## Steps
1. Create `tests/e2e/hermes/api/deliberation-auth.spec.ts`:
   - Test group: 'Unauthenticated requests'
     a. GET `/api/hermes/deliberations` without session → assert 401
     b. POST `/api/hermes/deliberations` without session → assert 401
   - Test group: 'Read-only user (hermes:read claim only)'
     a. Load `readOnlyUser` storageState
     b. GET `/api/hermes/deliberations` → assert 200
     c. GET `/api/hermes/deliberations/{id}` → assert 200
     d. POST `/api/hermes/deliberations` → assert 403
   - Test group: 'Full access user (hermes:read + hermes:trigger)'
     a. Load `fullAccessUser` storageState
     b. GET `/api/hermes/deliberations` → assert 200
     c. POST `/api/hermes/deliberations` → assert 201
   - Test group: 'Admin endpoints'
     a. Non-admin user hits `/api/hermes/admin/migrate-artifacts` → assert 403
2. Each test should be self-contained and not depend on ordering.
3. Use `test.describe` blocks for each persona grouping.

## Validation
Run `npx playwright test tests/e2e/hermes/api/deliberation-auth.spec.ts` against staging. All 7+ auth scenarios pass with correct HTTP status codes. No false positives (verify a broken session actually returns 401).