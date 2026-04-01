## Acceptance Criteria

- [ ] 1. Component test: render DesignSnapshotList with mocked API returning 3 PRs; verify 3 cards appear with correct titles and status badges. 2. Component test: render with empty PR array; verify 'No design snapshots' message displays. 3. Component test: click a PR card; verify DesignDeltaViewer renders with diff content. 4. Accessibility audit: run axe-core on the page; zero critical or serious violations. 5. Responsive test: capture screenshots at 320px, 768px, and 1920px widths; verify no horizontal overflow and cards stack/grid appropriately. 6. Error state test: mock a 500 response; verify error message displays and no uncaught exceptions.

## Verification Notes

- [ ] Confirm dependencies are satisfied before implementation.
- [ ] Update tests, docs, and configuration touched by this task.
- [ ] Validate the final behavior against the task objective.