Implement task 9: Validate Design Snapshot PR Surfacing in Frontend (Tess - Test frameworks)

## Goal
Validate that the Design Snapshot PR dashboard component (Task 4) correctly surfaces PR metadata, status, and links from a completed pipeline run. Since no design artifacts were supplied, focus on component structure, data correctness, and accessibility rather than visual matching.

## Task Context
- Agent owner: tess
- Stack: Test frameworks
- Priority: medium
- Dependencies: 4, 6

## Implementation Plan
1. Create test file `e2e/design-pr-surfacing.test.ts`. Use Playwright or the existing frontend E2E framework configured in the repository.
2. Test case 1 — Section visibility: navigate to the dashboard; assert the Design Snapshot PR section/tab is present and accessible via navigation.
3. Test case 2 — PR data display: after a pipeline run, navigate to the design PR section. Assert: at least one PR card is rendered with a visible title, status badge (open/merged/closed), repository name matching `5dlabs/sigma-1`, and a creation date.
4. Test case 3 — GitHub link: assert the PR card contains a link matching `https://github.com/5dlabs/sigma-1/pull/\d+` that opens in a new tab (`target="_blank"`).
5. Test case 4 — Task scaffold list: click into or expand a PR detail view. Assert at least one scaffold file is listed with a filename.
6. Test case 5 — Empty state: mock or navigate when no PRs exist. Assert the empty state message "No design snapshot PRs found" (or equivalent) is displayed.
7. Test case 6 — Accessibility: run axe-core against the design PR section. Assert zero critical or serious accessibility violations.
8. Test case 7 — Pipeline scoping: assert the section only shows PRs for the current pipeline run (pass `pipelineRunId` via URL param and verify no cross-run data leaks).

## Acceptance Criteria
1. Design PR section is reachable from the dashboard navigation. 2. After a pipeline run, at least one PR card renders with: title (non-empty string), status badge, repo name '5dlabs/sigma-1', and ISO-formatted date. 3. GitHub PR link matches expected URL pattern and has `target="_blank"`. 4. PR detail view lists >= 1 scaffold file. 5. Empty state renders correct message when no PRs exist. 6. axe-core accessibility scan returns zero critical/serious violations. 7. Displayed PRs are scoped to the specified pipelineRunId.

## Subtasks
- Set up E2E test file and navigation/section visibility tests: Create the Playwright test file `e2e/design-pr-surfacing.test.ts`, configure any required test fixtures (base URL, authentication/session), and implement Test Case 1 — verifying the Design Snapshot PR section is present and reachable from the dashboard navigation.
- Implement PR data display, GitHub link, and scaffold list validation tests: Add Test Cases 2, 3, and 4 to the E2E test file — validating that after a pipeline run, PR cards render correct metadata (title, status badge, repo name, date), contain properly-formed GitHub links opening in new tabs, and that expanding a PR detail view lists scaffold files.
- Implement empty state, accessibility scan, and pipeline scoping tests: Add Test Cases 5, 6, and 7 — verifying the empty state message when no PRs exist, running an axe-core accessibility audit with zero critical/serious violations, and asserting PR data is scoped to the current pipelineRunId with no cross-run leakage.

## Deliverables
- Update the relevant code, configuration, and tests.
- Keep artifacts aligned with the acceptance criteria.
- Document blockers or assumptions in your final summary.