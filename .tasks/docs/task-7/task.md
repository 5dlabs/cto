## Automate E2E Testing for Hermes Intake Pipeline (Tess - Test frameworks)

### Objective
Create a comprehensive E2E test suite using Playwright that validates the entire Hermes intake pipeline — from API deliberation triggering through artifact generation to frontend surfacing — covering both API-level and browser-level tests against the staging environment.

### Ownership
- Agent: tess
- Stack: Playwright
- Priority: high
- Status: pending
- Dependencies: 1, 2, 3, 4, 5, 6

### Implementation Details
Step-by-step implementation:

1. **Project setup:** Initialize Playwright test project:
   - `tests/e2e/hermes/` directory structure
   - `playwright.config.ts` with:
     - `baseURL` pointing to staging environment (from env var `E2E_BASE_URL`)
     - Projects: `chromium`, `firefox`, `webkit` for cross-browser
     - API project using `request` context (no browser)
     - Retries: 2 for CI, 0 for local
     - Reporter: HTML + JUnit (for CI integration)
   - Global setup: authenticate a test session with `hermes:read` and `hermes:trigger` claims

2. **API test suite** (`tests/e2e/hermes/api/`):
   - `deliberation-lifecycle.spec.ts`:
     a. POST create deliberation → assert 201, response contains UUID
     b. GET deliberation by ID → assert status transitions (pending → processing → completed)
     c. GET deliberation artifacts → assert at least 1 current_site_screenshot and 1 variant_snapshot
     d. GET artifact presigned URL → assert 200, URL resolves to image/png
   - `deliberation-auth.spec.ts`:
     a. Request without session → 401
     b. Request with session but without `hermes:trigger` → 403 on POST
     c. Request with `hermes:read` only → 200 on GET, 403 on POST
   - `deliberation-pagination.spec.ts`:
     a. Create 15 deliberations, request page 1 (limit 10) → 10 results + pagination metadata
     b. Request page 2 → 5 results
   - `artifact-migration.spec.ts`:
     a. Trigger migration via admin endpoint → 202
     b. Poll until complete → verify artifact counts match

3. **Browser test suite** (`tests/e2e/hermes/browser/`):
   - `environment-banner.spec.ts`:
     a. Navigate to staging URL → assert banner with 'STAGING' text is visible
     b. Assert banner has amber-family background color
   - `deliberation-dashboard.spec.ts`:
     a. Navigate to `/hermes` → assert deliberation cards render with correct count
     b. Click a deliberation card → navigates to detail page
     c. Assert loading skeleton appears before data loads
   - `artifact-comparison.spec.ts`:
     a. Navigate to a completed deliberation → assert comparison view shows current-site screenshot
     b. Click variant thumbnail → assert variant image loads in comparison view
     c. Assert both images have non-zero dimensions
   - `artifact-viewer.spec.ts`:
     a. Click artifact → dialog opens with full-size image
     b. Assert download button is present and functional
     c. Press Escape → dialog closes
   - `feature-flag.spec.ts`:
     a. With `HERMES_ENABLED=false`, navigate to `/hermes` → assert redirect or 404
     b. Assert Hermes nav item is not in DOM
   - `accessibility.spec.ts`:
     a. Run `@axe-core/playwright` on `/hermes` → zero critical/serious violations
     b. Run on `/hermes/[id]` → zero critical/serious violations

4. **CI integration:**
   - GitHub Actions workflow: `.github/workflows/hermes-e2e.yml`
   - Triggers: on PR to main, on push to staging branch
   - Runs against staging environment using in-cluster runners (`actions.github.com`)
   - Uploads Playwright HTML report and trace files as artifacts
   - Exit code gates ArgoCD staging → production promotion (Task 1 ArgoCD hook)

5. **Test data management:**
   - Global setup creates test deliberations and artifacts needed by browser tests
   - Global teardown cleans up test data
   - Tests are parallelizable — no shared mutable state between test files

6. **Flakiness mitigation:**
   - Use `page.waitForSelector` and `expect.toBeVisible()` instead of fixed delays
   - API tests poll with timeout for async operations (deliberation completion)
   - All screenshot comparisons use Playwright's visual regression with 1% threshold

### Subtasks
- [ ] Initialize Playwright project structure and configuration: Set up the Playwright test project directory structure, install dependencies, and create playwright.config.ts with cross-browser projects, API request context, retry policy, and reporter configuration.
- [ ] Implement global setup with authentication and test data seeding: Create the global setup script that authenticates test sessions with the required RBAC claims (hermes:read, hermes:trigger) and seeds the test data (deliberations and artifacts) needed by all test suites.
- [ ] Implement API test suite for deliberation lifecycle, pagination, and migration: Create API-level spec files covering the full deliberation lifecycle (create → status transitions → artifacts → presigned URLs), pagination behavior, and artifact migration endpoint.
- [ ] Implement API test suite for authentication and authorization: Create API-level spec files that validate all authentication and authorization scenarios: unauthenticated requests, insufficient claims, and correct claims.
- [ ] Implement browser test suite for environment banner, dashboard, and feature flag: Create Playwright browser specs covering the staging environment banner display, deliberation dashboard rendering and navigation, and feature flag gating behavior.
- [ ] Implement browser test suite for artifact comparison, artifact viewer, and accessibility: Create Playwright browser specs for the artifact comparison view, artifact viewer dialog interaction, and axe-core accessibility audits on Hermes pages.
- [ ] Create GitHub Actions CI workflow for Hermes E2E tests: Create the GitHub Actions workflow file that runs the full E2E test suite against the staging environment on PR and push triggers, uploads reports, and gates ArgoCD production promotion.