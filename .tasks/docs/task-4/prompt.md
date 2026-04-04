Implement task 4: Implement Design Snapshot PR Surfacing Dashboard Component (Blaze - React/Next.js)

## Goal
Build a dedicated dashboard section in the existing Next.js web frontend that surfaces design snapshot PRs created by the pipeline. The section displays PR metadata (title, status, repo, branch, created date), direct links to the GitHub PR, and the list of generated task scaffolds within the PR. Since no design artifacts were supplied (Stitch failed), follow the existing application's design system and component patterns.

## Task Context
- Agent owner: blaze
- Stack: React/Next.js
- Priority: medium
- Dependencies: 1

## Implementation Plan
1. Identify the existing dashboard layout structure in the Next.js app. Add a new route or section (e.g., `/dashboard/design-prs` or a tab within the existing dashboard) following established navigation patterns.
2. Create a `DesignSnapshotPRList` component that fetches PR data from the PM server API (endpoint to be provided by Task 2/6 — use `GET /api/pipeline/design-prs` as the expected contract). Display: PR title, status (open/merged/closed), target repo, branch name, creation timestamp, link to GitHub PR.
3. Create a `DesignSnapshotPRDetail` component showing the list of task scaffold files within the PR (fetched from the PR's file list via the API).
4. Handle loading, empty, and error states gracefully. Empty state: "No design snapshot PRs found for this pipeline run." Error state: show a retry button with the error message.
5. Use existing UI component library (buttons, cards, tables, status badges) from the application's design system. Do not introduce new external UI dependencies.
6. Ensure the component is accessible: semantic HTML, ARIA labels on interactive elements, keyboard navigable.
7. Add the navigation entry point (sidebar link, tab, or breadcrumb) consistent with other dashboard sections.
8. The component must support a `pipelineRunId` prop or URL param to scope PR display to a specific pipeline execution.

## Acceptance Criteria
1. Component renders without errors when given valid PR data (render test with mock data showing 2+ PRs). 2. Empty state renders correctly when API returns an empty array. 3. Error state renders with retry button when API call fails. 4. Each PR card displays title, status badge, repo name, branch, date, and a clickable GitHub link (assert on DOM elements). 5. `DesignSnapshotPRDetail` lists at least one scaffold file with filename and path. 6. Accessibility: axe-core scan of the rendered component returns zero violations. 7. Navigation entry point is present in the dashboard sidebar/tabs.

## Subtasks
- Create the /dashboard/design-prs route and wire up navigation entry point: Add a new Next.js page route for the design snapshot PR section and integrate a navigation link (sidebar item, tab, or breadcrumb) into the existing dashboard layout consistent with other sections.
- Implement data-fetching hook for design snapshot PR API: Create a custom React hook (e.g., `useDesignSnapshotPRs`) that fetches PR list data from `GET /api/pipeline/design-prs` scoped by `pipelineRunId`, and a companion hook (`useDesignSnapshotPRDetail`) for fetching scaffold file listings for a single PR. Handle loading, error, and success states.
- Build DesignSnapshotPRList component with loading, empty, and error states: Create the `DesignSnapshotPRList` component that consumes the data-fetching hook and renders a list/table of PR cards with status badges, metadata, and GitHub links. Implement all three non-happy-path states: loading skeleton, empty state message, and error state with retry button.
- Build DesignSnapshotPRDetail component with scaffold file listing: Create the `DesignSnapshotPRDetail` component that displays the full metadata of a selected PR and lists all generated task scaffold files contained within it.
- Implement accessibility compliance across all design PR components: Audit and enhance the DesignSnapshotPRList and DesignSnapshotPRDetail components for full accessibility: semantic HTML, ARIA labels, keyboard navigation, and focus management.

## Deliverables
- Update the relevant code, configuration, and tests.
- Keep artifacts aligned with the acceptance criteria.
- Document blockers or assumptions in your final summary.