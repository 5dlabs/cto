Implement task 8: Display Design Snapshot PRs in Web Frontend (Blaze - React/Next.js)

## Goal
Add a PR status section to the pipeline dashboard that displays the design snapshot PR created by the pipeline, including PR URL, status, file count, and a link to the GitHub PR. Contingent on D5 resolution.

## Task Context
- Agent owner: blaze
- Stack: React/Next.js
- Priority: low
- Dependencies: 4, 6

## Implementation Plan
1. Create a `SnapshotPR` component using shadcn/ui Card that displays: PR title, PR URL (as a clickable link opening in new tab), PR status (open, merged, closed) with color-coded badge, number of files changed, branch name.
2. Place the SnapshotPR component in the pipeline dashboard page, below the summary header and above the task list.
3. Fetch PR data from the PM server API endpoint that returns the PR creation result from Task 4.
4. Handle the case where no PR was created (GITHUB_TOKEN missing or API error): display a muted 'No snapshot PR created' message with the reason if available.
5. Add an external link icon next to the PR URL for visual affordance.
6. Write component tests for: PR present with open status, PR present with merged status, no PR created state.

## Acceptance Criteria
1. Component test: SnapshotPR rendered with a valid PR result displays the PR title, a clickable URL linking to `github.com/5dlabs/sigma-1/pull/N`, and an 'Open' badge in green. 2. Component test: SnapshotPR rendered with status='merged' displays a 'Merged' badge in purple. 3. Component test: SnapshotPR rendered with `pr_result=null` displays 'No snapshot PR created'. 4. Component test: PR URL link has `target='_blank'` and `rel='noopener noreferrer'` attributes. 5. Accessibility test: PR link is announced as an external link by screen readers.

## Subtasks
- Create SnapshotPR component with shadcn/ui Card and status badges: Build a SnapshotPR component using shadcn/ui Card that displays PR title, clickable URL with external link icon, color-coded status badge (open=green, merged=purple, closed=red), file count, and branch name.
- Integrate SnapshotPR into pipeline dashboard page with data fetching: Place the SnapshotPR component in the pipeline dashboard page below the summary header and above the task list, fetching PR data from the PM server API endpoint.
- Write component and accessibility tests for SnapshotPR: Write comprehensive component tests covering all PR display states (open, merged, closed, null) and accessibility requirements for external links.

## Deliverables
- Update the relevant code, configuration, and tests.
- Keep artifacts aligned with the acceptance criteria.
- Document blockers or assumptions in your final summary.