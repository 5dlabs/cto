## Show Pipeline Status Notifications in Web Frontend (Blaze - React/Next.js)

### Objective
Add a real-time or polling-based pipeline status indicator and notification timeline to the dashboard, showing pipeline lifecycle events (start, complete, error) and their timestamps. Displays the same events that are dispatched to Discord/Linear bridges. Contingent on D5 resolution.

### Ownership
- Agent: blaze
- Stack: React/Next.js
- Priority: low
- Status: pending
- Dependencies: 5, 6

### Implementation Details
1. Create a `PipelineStatus` component using shadcn/ui Badge for the current status (running, complete, error) with color coding: blue=running, green=complete, red=error.
2. Create a `NotificationTimeline` component that displays a chronological list of pipeline events: each event shows type (start/complete/error), timestamp, and brief description.
3. Implement polling (every 5 seconds) against the PM server API to fetch current pipeline status and event history. Use SWR or React Query for data fetching with automatic revalidation.
4. Place PipelineStatus in the dashboard header (next to the summary counts from Task 6). Place NotificationTimeline in a sidebar or below the task list.
5. When pipeline status is 'complete', show links to: Linear session, GitHub PR (from Task 4), and indicate that Discord/Linear notifications were sent.
6. When pipeline status is 'error', display the error message prominently with a red background alert using shadcn/ui Alert component.
7. Write component tests for: each status state rendering, timeline event ordering, polling behavior, error alert display.

### Subtasks
- [ ] Implement PipelineStatus badge component with three states and error Alert: Create the PipelineStatus component using shadcn/ui Badge that renders color-coded status indicators (blue=running, green=complete, red=error). When status is 'complete', render links to Linear session and GitHub PR. When status is 'error', render a shadcn/ui Alert component with role='alert' and the error message on a red background.
- [ ] Implement NotificationTimeline component with polling data fetching: Create the NotificationTimeline component that displays a chronological list of pipeline events (start/complete/error with timestamps and descriptions). Implement SWR or React Query polling hook with 5-second revalidation interval against the PM server API to fetch pipeline status and event history. Wire both PipelineStatus and NotificationTimeline into the dashboard layout.
- [ ] Write comprehensive component and accessibility tests for pipeline status UI: Write full test suite covering all PipelineStatus states, NotificationTimeline rendering and ordering, polling behavior verification, error alert display, and accessibility compliance for the error alert.