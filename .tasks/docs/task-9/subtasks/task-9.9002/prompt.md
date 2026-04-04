Implement subtask 9002: Implement NotificationTimeline component with polling data fetching

## Objective
Create the NotificationTimeline component that displays a chronological list of pipeline events (start/complete/error with timestamps and descriptions). Implement SWR or React Query polling hook with 5-second revalidation interval against the PM server API to fetch pipeline status and event history. Wire both PipelineStatus and NotificationTimeline into the dashboard layout.

## Steps
1. Create a data-fetching hook `src/hooks/usePipelineStatus.ts` using SWR (or React Query) that calls the PM server API endpoint (e.g., `GET /api/pipeline/status`). Configure `refreshInterval: 5000` for 5-second polling. The hook should return `{ status, events, error, isLoading }`.
2. Define TypeScript types: `PipelineEvent { type: 'start' | 'complete' | 'error'; timestamp: string; description: string }` and `PipelineStatusResponse { status: 'running' | 'complete' | 'error'; errorMessage?: string; linearSessionUrl?: string; prUrl?: string; events: PipelineEvent[] }`.
3. Create `src/components/NotificationTimeline.tsx`. Accept props: `events: PipelineEvent[]`. Render a vertical timeline list sorted chronologically (newest first or oldest first — follow the existing dashboard design convention). Each item shows: event type as a labeled icon/badge, formatted timestamp, and description text.
4. Create a container component `src/components/PipelineStatusPanel.tsx` that uses the `usePipelineStatus` hook, passes data to `PipelineStatus` (in dashboard header) and `NotificationTimeline` (in sidebar or below task list).
5. Integrate PipelineStatusPanel into the existing dashboard page layout from Task 6: PipelineStatus in the header row, NotificationTimeline in a collapsible sidebar or section below the task list.

## Validation
Component test: Render NotificationTimeline with 3 mock events and assert they appear in chronological order with correct type labels and formatted timestamps. Integration test: Mock the PM server API, render PipelineStatusPanel, advance timers by 5 seconds using fake timers, and assert that SWR/React Query re-fetches and the display updates with new data.