Implement subtask 9003: Implement empty state, accessibility scan, and pipeline scoping tests

## Objective
Add Test Cases 5, 6, and 7 — verifying the empty state message when no PRs exist, running an axe-core accessibility audit with zero critical/serious violations, and asserting PR data is scoped to the current pipelineRunId with no cross-run leakage.

## Steps
1. Test Case 5 — Empty state:
   a. Mock the API response to return an empty PR list (use Playwright `page.route` to intercept the PR endpoint and respond with `{ prs: [] }`).
   b. Navigate to the Design Snapshot PR section.
   c. Assert the empty state element is visible and contains text matching 'No design snapshot PRs found' (case-insensitive).
   d. Restore the original route after the test.
2. Test Case 6 — Accessibility:
   a. Install/import `@axe-core/playwright` (or use `axe-playwright`).
   b. Navigate to the Design Snapshot PR section with data present.
   c. Run `new AxeBuilder({ page }).include('[data-testid="design-pr-section"]').analyze()`.
   d. Filter results for violations with impact `critical` or `serious`.
   e. Assert the filtered violations array has length 0; if not, log violation details for debugging.
3. Test Case 7 — Pipeline scoping:
   a. Navigate to the dashboard with a specific `pipelineRunId` query param (e.g., `?pipelineRunId=run-abc-123`).
   b. Collect all displayed PR card elements and extract their associated pipelineRunId data attributes or API-sourced metadata.
   c. Assert every PR card's pipelineRunId matches `run-abc-123`.
   d. Navigate with a different `pipelineRunId` (e.g., `run-xyz-456`) and assert the previously displayed PRs are NOT shown (no cross-run data leak).

## Validation
Run `npx playwright test e2e/design-pr-surfacing.test.ts --grep 'empty state|accessibility|pipeline scoping'`. Empty state test passes when mocked empty response triggers correct message. Accessibility test passes with zero critical/serious axe violations. Pipeline scoping test passes when switching pipelineRunId shows only run-specific PRs with no leakage.