Implement subtask 7005: Implement browser test suite for environment banner, dashboard, and feature flag

## Objective
Create Playwright browser specs covering the staging environment banner display, deliberation dashboard rendering and navigation, and feature flag gating behavior.

## Steps
1. Create `tests/e2e/hermes/browser/environment-banner.spec.ts`:
   - Navigate to `E2E_BASE_URL`
   - Assert an element containing 'STAGING' text is visible
   - Assert the banner element's background color is in the amber family (e.g., rgb values in amber/yellow range)
   - Use `page.waitForSelector` for reliability

2. Create `tests/e2e/hermes/browser/deliberation-dashboard.spec.ts`:
   - Load `fullAccessUser` storageState for browser context
   - Navigate to `/hermes`
   - Assert loading skeleton is visible initially (use `{ timeout: 2000 }` for the assertion, accept if skeleton is briefly shown)
   - Assert deliberation cards render; count matches API response from GET `/api/hermes/deliberations`
   - Click the first deliberation card → assert URL changes to `/hermes/{id}` pattern
   - Assert detail page loads with deliberation data (title, status, created date visible)

3. Create `tests/e2e/hermes/browser/feature-flag.spec.ts`:
   - Test: With feature disabled (requires environment configuration per decision point), navigate to `/hermes` → assert either redirect to home/404 page
   - Test: Assert the Hermes navigation item is NOT present in the sidebar/nav DOM
   - Note: If dynamic flag toggling is not feasible, these tests should be skipped with `test.skip` and a clear annotation explaining the dependency on a separate deployment.

4. All browser tests should use `expect(locator).toBeVisible()` pattern, never `page.waitForTimeout()`.

## Validation
Run browser specs across all 3 browser projects (chromium, firefox, webkit). Banner test confirms STAGING text and amber background. Dashboard test renders correct card count matching API. Feature flag test confirms Hermes is inaccessible when disabled. All pass in all 3 browsers.