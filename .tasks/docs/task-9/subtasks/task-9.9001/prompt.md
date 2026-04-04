Implement subtask 9001: Set up E2E test file and navigation/section visibility tests

## Objective
Create the Playwright test file `e2e/design-pr-surfacing.test.ts`, configure any required test fixtures (base URL, authentication/session), and implement Test Case 1 — verifying the Design Snapshot PR section is present and reachable from the dashboard navigation.

## Steps
1. Create `e2e/design-pr-surfacing.test.ts`. Import Playwright test utilities and any project-specific helpers.
2. Configure `beforeAll`/`beforeEach` hooks: launch browser context, navigate to the dashboard URL (e.g., `http://localhost:3000/dashboard`).
3. Implement test 'Design Snapshot PR section is visible': use `page.getByRole` or `page.locator` to find the section/tab labelled 'Design Snapshot' or equivalent.
4. Assert the section element `isVisible()`. If it is a tab, click it and assert the panel content container appears.
5. Verify keyboard navigability — tab to the section element and assert it receives focus.
6. Add `afterAll` teardown to close browser context.

## Validation
Run `npx playwright test e2e/design-pr-surfacing.test.ts --grep 'section is visible'`. The test passes when the Design Snapshot PR section/tab is located in the DOM, is visible, and can receive keyboard focus.