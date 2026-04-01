Implement subtask 4003: Implement DesignDeltaViewer component with diff library integration

## Objective
Build the `DesignDeltaViewer` component that fetches diff data for a selected PR from `GET /api/pipeline/:runId/prs/:number/diff` and renders it using `react-diff-viewer-continued` v3.x with side-by-side and inline toggle modes, plus its own loading and error states.

## Steps
1. Install `react-diff-viewer-continued` v3.x: `npm install react-diff-viewer-continued`.
2. Create `components/DesignDeltaViewer.tsx`. Accept props: `{ runId, prNumber, onClose }`.
3. Implement a hook `useDesignDiff(runId, prNumber)` that fetches `GET /api/pipeline/${runId}/prs/${prNumber}/diff`. Expect the response to contain `{ files: [{ filename, old_content, new_content }] }` (or adapt to the actual API shape).
4. **Loading state**: Show a skeleton or spinner while the diff is loading.
5. **Error state**: Show an error message with a 'Retry' button.
6. **Diff rendering**: For each file in the diff response, render a collapsible section with the filename as header. Inside, render `<ReactDiffViewer oldValue={old_content} newValue={new_content} splitView={splitView} />` where `splitView` is toggled by a button.
7. Add a toggle control at the top: 'Side-by-side' vs 'Inline' view mode.
8. Add a 'Close' button or back-navigation that calls `onClose`.
9. Style the viewer container to be full-width with horizontal scroll for wide diffs on narrow viewports.
10. Integrate into the page: when `onSelectPR` is called from DesignSnapshotList, show the DesignDeltaViewer (e.g., as a slide-in panel or replacing the list with a back button).

## Validation
Component test: render with mocked diff API returning 2 files, verify both file sections appear with diff content. Test the side-by-side/inline toggle changes the `splitView` prop. Test loading state renders spinner/skeleton. Test error state shows error message. Test close button calls onClose.