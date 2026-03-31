Implement subtask 7002: Implement global setup with authentication and test data seeding

## Objective
Create the global setup script that authenticates test sessions with the required RBAC claims (hermes:read, hermes:trigger) and seeds the test data (deliberations and artifacts) needed by all test suites.

## Steps
1. Create `tests/e2e/hermes/global-setup.ts`:
   - Authenticate against the staging auth endpoint to obtain session tokens for 3 personas:
     a. `fullAccessUser` — has both `hermes:read` and `hermes:trigger` claims
     b. `readOnlyUser` — has only `hermes:read` claim
     c. `unauthenticatedContext` — no session (for 401 tests)
   - Store auth state (cookies/tokens) in `tests/e2e/hermes/.auth/` directory using `storageState`
   - Seed test data via API:
     a. Create 15+ deliberations (needed for pagination tests)
     b. Ensure at least 1 deliberation reaches `completed` status with artifacts (poll with 60s timeout)
   - Store created resource IDs in a shared JSON file (`tests/e2e/hermes/.testdata/ids.json`) for consumption by test specs.
2. Create `tests/e2e/hermes/global-teardown.ts`:
   - Read `ids.json` and delete all test-created deliberations via API
   - Log cleanup results
3. Update `playwright.config.ts` projects to use the appropriate `storageState` files for each auth persona.

## Validation
Run the global setup in isolation (`npx playwright test --global-setup-only` or invoke directly). Verify `.auth/` contains valid storageState JSON files for each persona, `.testdata/ids.json` contains at least 15 deliberation UUIDs, and at least 1 deliberation has status 'completed' when queried via API.