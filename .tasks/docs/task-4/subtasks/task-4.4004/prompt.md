Implement subtask 4004: Build DesignSnapshotPRDetail component with scaffold file listing

## Objective
Create the `DesignSnapshotPRDetail` component that displays the full metadata of a selected PR and lists all generated task scaffold files contained within it.

## Steps
1. Create `components/DesignSnapshotPRDetail.tsx`. It accepts a `prId` prop and calls `useDesignSnapshotPRDetail(prId)`.
2. Display full PR metadata at the top: title, status, repo, branch, created date, GitHub link (same layout as list card but expanded).
3. Below metadata, render a 'Scaffold Files' section as a list or table. Each row shows:
   - File name (e.g., `task-001-setup-db.md`)
   - File path within the PR (e.g., `.pipeline/scaffolds/task-001-setup-db.md`)
4. Handle loading state (skeleton), empty file list ('No scaffold files found in this PR.'), and error state (retry button).
5. Use the existing design system's list/table components. Files should be visually distinct items (e.g., monospace font for filenames).
6. Integrate this component into the page created in 4001 — either as a drill-down route (`/dashboard/design-prs/[prId]`) or as an expandable panel within the list, depending on the app's existing detail-view pattern.

## Validation
Render test with mock PR detail data: (1) PR metadata (title, status, repo, branch, date, GitHub link) renders correctly. (2) At least one scaffold file with filename and path is listed. (3) Empty scaffold file list shows the empty-state message. (4) Error state shows retry button.