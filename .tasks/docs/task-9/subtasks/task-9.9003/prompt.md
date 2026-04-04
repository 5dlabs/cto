Implement subtask 9003: Write comprehensive component and accessibility tests for pipeline status UI

## Objective
Write full test suite covering all PipelineStatus states, NotificationTimeline rendering and ordering, polling behavior verification, error alert display, and accessibility compliance for the error alert.

## Steps
1. Create `src/components/__tests__/PipelineStatus.test.tsx`:
   - Test: status='running' renders blue 'Running' badge (check className or data attribute for blue variant).
   - Test: status='complete' renders green 'Complete' badge and renders Linear session link and PR link with correct href values.
   - Test: status='error' renders red Alert with the exact error message text visible in the DOM.
   - Test: status='error' Alert element has `role='alert'` attribute.
2. Create `src/components/__tests__/NotificationTimeline.test.tsx`:
   - Test: 3 events passed as props render 3 timeline items.
   - Test: Events appear in correct chronological order (verify DOM order matches timestamp sort).
   - Test: Each event displays its type label, formatted timestamp, and description.
   - Test: Empty events array renders an empty state or no items.
3. Create `src/hooks/__tests__/usePipelineStatus.test.ts` or integration test:
   - Test: Mock API with MSW or jest mock. Render component using the hook. Assert initial data loads.
   - Test: Use fake timers, advance 5 seconds, assert re-fetch occurs and updated data renders.
4. Run all tests with `npm test` or `vitest` and verify 100% of the above cases pass.

## Validation
All specified tests pass: 3 PipelineStatus state tests, 1 accessibility test (role='alert'), 4 NotificationTimeline tests (rendering, ordering, labels, empty state), and 2 polling integration tests (initial load, 5-second revalidation). Run via test runner with coverage report showing PipelineStatus.tsx, NotificationTimeline.tsx, and usePipelineStatus.ts covered.