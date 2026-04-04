## Implement Design Snapshot PR Surfacing Dashboard Component (Blaze - React/Next.js)

### Objective
Build a dedicated dashboard section in the existing Next.js web frontend that surfaces design snapshot PRs created by the pipeline. The section displays PR metadata (title, status, repo, branch, created date), direct links to the GitHub PR, and the list of generated task scaffolds within the PR. Since no design artifacts were supplied (Stitch failed), follow the existing application's design system and component patterns.

### Ownership
- Agent: blaze
- Stack: React/Next.js
- Priority: medium
- Status: pending
- Dependencies: 1

### Implementation Details
1. Identify the existing dashboard layout structure in the Next.js app. Add a new route or section (e.g., `/dashboard/design-prs` or a tab within the existing dashboard) following established navigation patterns.
2. Create a `DesignSnapshotPRList` component that fetches PR data from the PM server API (endpoint to be provided by Task 2/6 — use `GET /api/pipeline/design-prs` as the expected contract). Display: PR title, status (open/merged/closed), target repo, branch name, creation timestamp, link to GitHub PR.
3. Create a `DesignSnapshotPRDetail` component showing the list of task scaffold files within the PR (fetched from the PR's file list via the API).
4. Handle loading, empty, and error states gracefully. Empty state: "No design snapshot PRs found for this pipeline run." Error state: show a retry button with the error message.
5. Use existing UI component library (buttons, cards, tables, status badges) from the application's design system. Do not introduce new external UI dependencies.
6. Ensure the component is accessible: semantic HTML, ARIA labels on interactive elements, keyboard navigable.
7. Add the navigation entry point (sidebar link, tab, or breadcrumb) consistent with other dashboard sections.
8. The component must support a `pipelineRunId` prop or URL param to scope PR display to a specific pipeline execution.

### Subtasks
- [ ] Create the /dashboard/design-prs route and wire up navigation entry point: Add a new Next.js page route for the design snapshot PR section and integrate a navigation link (sidebar item, tab, or breadcrumb) into the existing dashboard layout consistent with other sections.
- [ ] Implement data-fetching hook for design snapshot PR API: Create a custom React hook (e.g., `useDesignSnapshotPRs`) that fetches PR list data from `GET /api/pipeline/design-prs` scoped by `pipelineRunId`, and a companion hook (`useDesignSnapshotPRDetail`) for fetching scaffold file listings for a single PR. Handle loading, error, and success states.
- [ ] Build DesignSnapshotPRList component with loading, empty, and error states: Create the `DesignSnapshotPRList` component that consumes the data-fetching hook and renders a list/table of PR cards with status badges, metadata, and GitHub links. Implement all three non-happy-path states: loading skeleton, empty state message, and error state with retry button.
- [ ] Build DesignSnapshotPRDetail component with scaffold file listing: Create the `DesignSnapshotPRDetail` component that displays the full metadata of a selected PR and lists all generated task scaffold files contained within it.
- [ ] Implement accessibility compliance across all design PR components: Audit and enhance the DesignSnapshotPRList and DesignSnapshotPRDetail components for full accessibility: semantic HTML, ARIA labels, keyboard navigation, and focus management.