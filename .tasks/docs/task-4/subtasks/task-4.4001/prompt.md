Implement subtask 4001: Create page route and data-fetching hook for design snapshot PRs

## Objective
Set up the Next.js dynamic page route at `/pipeline/[runId]/design-snapshots` and implement a custom hook `useDesignSnapshotPRs(runId)` that fetches PR data from `GET /api/pipeline/:runId/prs`, returning loading, error, and data states.

## Steps
1. Create the file `app/pipeline/[runId]/design-snapshots/page.tsx` (or `pages/pipeline/[runId]/design-snapshots.tsx` depending on the project's Next.js routing convention).
2. Extract `runId` from the route params.
3. Implement `useDesignSnapshotPRs(runId)` in a `hooks/` directory. Use `fetch` or the project's existing HTTP client to call `GET /api/pipeline/${runId}/prs`. Return `{ data, isLoading, error }` shape.
4. Handle non-200 responses by throwing or returning an error object.
5. The page component should call this hook and pass state down to child components (built in subsequent subtasks). For now, render a placeholder wrapper `<div>` that conditionally renders based on `isLoading`, `error`, and `data`.
6. Use Tailwind CSS for the page layout container.

## Validation
Unit test the hook with msw or a fetch mock: verify it returns loading=true initially, resolves to data on 200, and returns an error object on 500. Verify the page route renders without crashing when given a valid runId.