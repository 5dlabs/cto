Implement subtask 7003: Implement API test suite for deliberation lifecycle, pagination, and migration

## Objective
Create API-level spec files covering the full deliberation lifecycle (create → status transitions → artifacts → presigned URLs), pagination behavior, and artifact migration endpoint.

## Steps
1. Create `tests/e2e/hermes/api/deliberation-lifecycle.spec.ts`:
   - Use the `api` project (request context, no browser)
   - Load `fullAccessUser` storageState
   - Test: POST `/api/hermes/deliberations` → assert 201, body has `id` (UUID format), `status: 'pending'`
   - Test: Poll GET `/api/hermes/deliberations/{id}` with 5s intervals, max 60s → assert status transitions from `pending` to `processing` to `completed`
   - Test: GET `/api/hermes/deliberations/{id}/artifacts` → assert array length >= 2, includes type `current_site_screenshot` and `variant_snapshot`
   - Test: GET artifact presigned URL from artifact object → assert 200, `Content-Type: image/png`
2. Create `tests/e2e/hermes/api/deliberation-pagination.spec.ts`:
   - Use the seeded 15+ deliberations from global setup
   - Test: GET `/api/hermes/deliberations?page=1&limit=10` → assert 10 results, response has `total`, `page`, `limit`, `totalPages` metadata
   - Test: GET page 2 → assert 5 results (for exactly 15 seeded)
   - Test: GET with limit > total → assert all results returned
3. Create `tests/e2e/hermes/api/artifact-migration.spec.ts`:
   - Test: POST `/api/hermes/admin/migrate-artifacts` → assert 202
   - Test: Poll migration status endpoint until complete → verify artifact counts match expected
4. Implement polling helper utility in `tests/e2e/hermes/utils/poll.ts` with configurable interval and timeout.

## Validation
Run `npx playwright test tests/e2e/hermes/api/deliberation-lifecycle.spec.ts tests/e2e/hermes/api/deliberation-pagination.spec.ts tests/e2e/hermes/api/artifact-migration.spec.ts` against staging. All tests pass. Lifecycle test completes within 90s. Pagination returns correct counts. Presigned URL resolves to a valid PNG.