## Acceptance Criteria

- [ ] 1. Component test: PipelineStatus with status='running' renders a blue 'Running' badge. 2. Component test: PipelineStatus with status='complete' renders a green 'Complete' badge and displays Linear session and PR links. 3. Component test: PipelineStatus with status='error' renders a red Alert component with the error message text visible. 4. Component test: NotificationTimeline with 3 events renders them in chronological order with correct timestamps and event type labels. 5. Integration test: Mock the PM server API with SWR/React Query; verify that the component re-fetches status after the 5-second polling interval and updates the display. 6. Accessibility test: Error alert has role='alert' for screen reader announcement.

## Verification Notes

- [ ] Confirm dependencies are satisfied before implementation.
- [ ] Update tests, docs, and configuration touched by this task.
- [ ] Validate the final behavior against the task objective.