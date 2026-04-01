## Implement Design Snapshot PR Surfacing (Blaze - React/Next.js)

### Objective
Build a web frontend component that surfaces design snapshot PRs generated during the pipeline, allowing users to view PR details, design deltas, and navigate to the GitHub PR. This addresses the hasFrontend=true, targets=web requirement from the design intake.

### Ownership
- Agent: blaze
- Stack: React/Next.js
- Priority: medium
- Status: pending
- Dependencies: 1, 7

### Implementation Details
1. Create a new page route `/pipeline/[runId]/design-snapshots` in the Next.js app.
2. Implement a `DesignSnapshotList` component that fetches PR data from the PM server endpoint `GET /api/pipeline/:runId/prs` which returns `{ prs: [{ number, title, url, status, files_changed, created_at }] }`.
3. For each PR, render a card showing:
   - PR title and number as a link to GitHub.
   - Status badge (open/merged/closed).
   - Number of files changed.
   - Created timestamp in relative format.
4. Implement a `DesignDeltaViewer` component that, when a PR card is clicked, fetches diff data from `GET /api/pipeline/:runId/prs/:number/diff` and renders a side-by-side or inline diff view using a lightweight diff library (e.g., `react-diff-viewer-continued` v3.x).
5. Add accessibility: all interactive elements must have ARIA labels, cards are keyboard-navigable, diff viewer supports screen readers.
6. Add loading skeletons and error states for all async fetches.
7. Ensure the page is responsive for viewport widths from 320px to 1920px.
8. Since stitch_status=failed, do not rely on any Stitch-generated candidates; build components from scratch following the existing project's design system if one exists, or use Tailwind CSS utility classes.

### Subtasks
- [ ] Create page route and data-fetching hook for design snapshot PRs: Set up the Next.js dynamic page route at `/pipeline/[runId]/design-snapshots` and implement a custom hook `useDesignSnapshotPRs(runId)` that fetches PR data from `GET /api/pipeline/:runId/prs`, returning loading, error, and data states.
- [ ] Build DesignSnapshotList component with PR cards, loading skeletons, and error state: Implement the `DesignSnapshotList` component that receives PR data (or loading/error states) and renders a list of PR cards with title, number, GitHub link, status badge, files changed count, and relative timestamp. Include loading skeleton placeholders and an error message UI.
- [ ] Implement DesignDeltaViewer component with diff library integration: Build the `DesignDeltaViewer` component that fetches diff data for a selected PR from `GET /api/pipeline/:runId/prs/:number/diff` and renders it using `react-diff-viewer-continued` v3.x with side-by-side and inline toggle modes, plus its own loading and error states.
- [ ] Accessibility pass: ARIA labels, keyboard navigation, and screen reader support: Audit and enhance all components built in prior subtasks for accessibility compliance: add ARIA attributes, ensure full keyboard navigation, and verify screen reader compatibility for the PR card list and diff viewer.
- [ ] Responsive layout implementation for 320px to 1920px viewports: Ensure the entire design-snapshots page, including the PR card grid and the diff viewer, is fully responsive across viewports from 320px to 1920px with no horizontal overflow or layout breakage.
- [ ] Component and integration tests for all UI states and accessibility audit: Write comprehensive component tests covering all states of DesignSnapshotList and DesignDeltaViewer (populated, empty, loading, error, interaction), plus an automated axe-core accessibility audit and responsive snapshot tests.