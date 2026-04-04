Implement task 9: Show Pipeline Status Notifications in Web Frontend (Blaze - React/Next.js)

## Goal
Add a real-time or polling-based pipeline status indicator and notification timeline to the dashboard, showing pipeline lifecycle events (start, complete, error) and their timestamps. Displays the same events that are dispatched to Discord/Linear bridges. Contingent on D5 resolution.

## Task Context
- Agent owner: blaze
- Stack: React/Next.js
- Priority: low
- Dependencies: 5, 6

## Implementation Plan
1. Create a `PipelineStatus` component using shadcn/ui Badge for the current status (running, complete, error) with color coding: blue=running, green=complete, red=error.
2. Create a `NotificationTimeline` component that displays a chronological list of pipeline events: each event shows type (start/complete/error), timestamp, and brief description.
3. Implement polling (every 5 seconds) against the PM server API to fetch current pipeline status and event history. Use SWR or React Query for data fetching with automatic revalidation.
4. Place PipelineStatus in the dashboard header (next to the summary counts from Task 6). Place NotificationTimeline in a sidebar or below the task list.
5. When pipeline status is 'complete', show links to: Linear session, GitHub PR (from Task 4), and indicate that Discord/Linear notifications were sent.
6. When pipeline status is 'error', display the error message prominently with a red background alert using shadcn/ui Alert component.
7. Write component tests for: each status state rendering, timeline event ordering, polling behavior, error alert display.

## Acceptance Criteria
1. Component test: PipelineStatus with status='running' renders a blue 'Running' badge. 2. Component test: PipelineStatus with status='complete' renders a green 'Complete' badge and displays Linear session and PR links. 3. Component test: PipelineStatus with status='error' renders a red Alert component with the error message text visible. 4. Component test: NotificationTimeline with 3 events renders them in chronological order with correct timestamps and event type labels. 5. Integration test: Mock the PM server API with SWR/React Query; verify that the component re-fetches status after the 5-second polling interval and updates the display. 6. Accessibility test: Error alert has role='alert' for screen reader announcement.

## Subtasks
- Implement PipelineStatus badge component with three states and error Alert: Create the PipelineStatus component using shadcn/ui Badge that renders color-coded status indicators (blue=running, green=complete, red=error). When status is 'complete', render links to Linear session and GitHub PR. When status is 'error', render a shadcn/ui Alert component with role='alert' and the error message on a red background.
- Implement NotificationTimeline component with polling data fetching: Create the NotificationTimeline component that displays a chronological list of pipeline events (start/complete/error with timestamps and descriptions). Implement SWR or React Query polling hook with 5-second revalidation interval against the PM server API to fetch pipeline status and event history. Wire both PipelineStatus and NotificationTimeline into the dashboard layout.
- Write comprehensive component and accessibility tests for pipeline status UI: Write full test suite covering all PipelineStatus states, NotificationTimeline rendering and ordering, polling behavior verification, error alert display, and accessibility compliance for the error alert.

## Deliverables
- Update the relevant code, configuration, and tests.
- Keep artifacts aligned with the acceptance criteria.
- Document blockers or assumptions in your final summary.