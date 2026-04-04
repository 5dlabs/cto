Implement subtask 9002: Implement PR data display, GitHub link, and scaffold list validation tests

## Objective
Add Test Cases 2, 3, and 4 to the E2E test file — validating that after a pipeline run, PR cards render correct metadata (title, status badge, repo name, date), contain properly-formed GitHub links opening in new tabs, and that expanding a PR detail view lists scaffold files.

## Steps
1. Test Case 2 — PR data display:
   a. Navigate to the Design Snapshot PR section (reuse navigation from 9001).
   b. Locate at least one PR card element (e.g., `[data-testid='pr-card']`).
   c. Assert card contains: a non-empty title string, a status badge element with text matching `/open|merged|closed/i`, a repo name element containing `5dlabs/sigma-1`, and a date element matching ISO format `/\d{4}-\d{2}-\d{2}/`.
2. Test Case 3 — GitHub link:
   a. Within the PR card, locate an anchor element.
   b. Assert `href` matches regex `https://github\.com/5dlabs/sigma-1/pull/\d+`.
   c. Assert `target` attribute equals `_blank`.
   d. Assert `rel` contains `noopener` (security best practice).
3. Test Case 4 — Task scaffold list:
   a. Click the PR card or its expand/detail button.
   b. Wait for the detail view to appear.
   c. Locate scaffold file list items; assert count >= 1.
   d. Assert each listed file has a non-empty filename string.

## Validation
Run `npx playwright test e2e/design-pr-surfacing.test.ts --grep 'PR data|GitHub link|scaffold'`. All three tests pass: PR card shows title, status, repo, date; link matches GitHub URL pattern with `target=_blank`; detail view lists at least one scaffold file.