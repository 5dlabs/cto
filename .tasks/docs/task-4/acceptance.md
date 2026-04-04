## Acceptance Criteria

- [ ] 1. Component renders without errors when given valid PR data (render test with mock data showing 2+ PRs). 2. Empty state renders correctly when API returns an empty array. 3. Error state renders with retry button when API call fails. 4. Each PR card displays title, status badge, repo name, branch, date, and a clickable GitHub link (assert on DOM elements). 5. `DesignSnapshotPRDetail` lists at least one scaffold file with filename and path. 6. Accessibility: axe-core scan of the rendered component returns zero violations. 7. Navigation entry point is present in the dashboard sidebar/tabs.

## Verification Notes

- [ ] Confirm dependencies are satisfied before implementation.
- [ ] Update tests, docs, and configuration touched by this task.
- [ ] Validate the final behavior against the task objective.