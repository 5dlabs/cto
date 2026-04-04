Implement subtask 4003: Build DesignSnapshotPRList component with loading, empty, and error states

## Objective
Create the `DesignSnapshotPRList` component that consumes the data-fetching hook and renders a list/table of PR cards with status badges, metadata, and GitHub links. Implement all three non-happy-path states: loading skeleton, empty state message, and error state with retry button.

## Steps
1. Create `components/DesignSnapshotPRList.tsx`. It accepts `pipelineRunId` as a prop and calls `useDesignSnapshotPRs(pipelineRunId)`.
2. **Loading state**: Render skeleton placeholders or a spinner using the existing design system's loading component.
3. **Empty state**: Render a centered message: 'No design snapshot PRs found for this pipeline run.' with an appropriate icon from the design system.
4. **Error state**: Render the error message text and a 'Retry' button that calls `retry()` from the hook.
5. **Populated state**: Render each PR as a card or table row using the existing card/table components. Each row displays:
   - PR title (as a heading or bold text)
   - Status badge (open=green, merged=purple, closed=red — use existing badge component)
   - Repository name
   - Branch name
   - Created date (formatted with locale-aware date formatting)
   - A clickable link/button opening the GitHub PR URL in a new tab (`target="_blank"`, `rel="noopener noreferrer"`)
6. Each PR card/row should be clickable to navigate to the detail view or expand to show scaffold files.
7. Do not introduce any new external UI dependencies.

## Validation
Render tests with mock data: (1) With 2+ PRs, all metadata fields (title, status badge, repo, branch, date, GitHub link) are present in the DOM. (2) With empty array, the empty-state message is rendered. (3) With a simulated fetch error, the error message and Retry button render; clicking Retry invokes the retry function. (4) GitHub links have correct href, target=_blank, and rel attributes.