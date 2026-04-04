Implement subtask 7007: Implement SWR-based polling with 5-second refresh interval

## Objective
Add automatic 5-second polling to the dashboard page using SWR's `refreshInterval` option. Display a 'Last updated' timestamp, and automatically disable polling when the pipeline status reaches 'complete' or 'failed'.

## Steps
1. Install SWR: `npm install swr`.
2. Create SWR hooks in `src/hooks/`:
   a. `usePipelineStatus()` — wraps `fetchPipelineStatus` with SWR, `refreshInterval: 5000`.
   b. `useDelegationStatus()` — wraps `fetchDelegationStatus` with SWR, `refreshInterval: 5000`.
3. Both hooks should:
   a. Accept a `paused` boolean option to disable polling.
   b. Return `{ data, error, isLoading, mutate }`.
4. In the dashboard page:
   a. Compute `isPipelineTerminal = status === 'complete' || status === 'failed'`.
   b. Pass `isPipelineTerminal` as the pause condition to both SWR hooks (use SWR's `refreshInterval: isPipelineTerminal ? 0 : 5000` pattern).
   c. Display a 'Last updated: {timestamp}' line below the banner, updated each time SWR re-fetches.
5. Add an inline shadcn/ui Alert (destructive variant) that appears when either SWR hook returns an error. Display the error message text — no modal dialogs.
6. Add a manual 'Refresh' button that calls `mutate()` on both hooks.

## Validation
With mock API returning changing data, dashboard updates automatically every 5 seconds without manual page refresh. 'Last updated' timestamp changes with each poll. When pipeline status is 'complete', polling stops (verify no further network requests in browser dev tools). When API returns 500, an inline Alert appears with the error message (not a modal). Manual refresh button triggers an immediate re-fetch.