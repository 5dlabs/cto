Implement subtask 4002: Implement data-fetching hook for design snapshot PR API

## Objective
Create a custom React hook (e.g., `useDesignSnapshotPRs`) that fetches PR list data from `GET /api/pipeline/design-prs` scoped by `pipelineRunId`, and a companion hook (`useDesignSnapshotPRDetail`) for fetching scaffold file listings for a single PR. Handle loading, error, and success states.

## Steps
1. Create `hooks/useDesignSnapshotPRs.ts` that accepts a `pipelineRunId` string parameter.
2. Use the project's existing data-fetching pattern (SWR, React Query, or plain fetch with useEffect) — do not introduce a new data-fetching library.
3. Call `GET /api/pipeline/design-prs?pipelineRunId={id}` and return `{ data: DesignSnapshotPR[], isLoading: boolean, error: Error | null, retry: () => void }`.
4. Define a TypeScript interface `DesignSnapshotPR` with fields: `id`, `title`, `status` ('open' | 'merged' | 'closed'), `repoName`, `branchName`, `createdAt` (ISO string), `githubUrl`, `scaffoldFiles?: ScaffoldFile[]`.
5. Define `ScaffoldFile` with fields: `filename`, `path`, `status`.
6. Create `hooks/useDesignSnapshotPRDetail.ts` that fetches scaffold file listing for a single PR by its ID.
7. Both hooks should expose a `retry` function that re-triggers the fetch for error recovery.

## Validation
Unit test the hook with msw or a fetch mock: (a) returns loading=true initially, (b) resolves with parsed PR data on success, (c) returns error object on network failure, (d) retry function re-invokes the fetch. TypeScript compiles with no type errors on the interfaces.