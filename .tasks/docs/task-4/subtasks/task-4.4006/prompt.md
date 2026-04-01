Implement subtask 4006: Component and integration tests for all UI states and accessibility audit

## Objective
Write comprehensive component tests covering all states of DesignSnapshotList and DesignDeltaViewer (populated, empty, loading, error, interaction), plus an automated axe-core accessibility audit and responsive snapshot tests.

## Steps
1. Use the project's testing framework (likely Vitest + React Testing Library or Jest + RTL).
2. **DesignSnapshotList tests**:
   - Render with 3 mocked PRs; assert 3 cards with correct title text, correct `href` on GitHub links, correct status badge text.
   - Render with empty `prs` array; assert 'No design snapshots' message.
   - Render with `isLoading=true`; assert skeleton elements with `aria-busy`.
   - Render with error; assert error message text and retry button. Click retry; assert callback called.
   - Click a card; assert `onSelectPR` called with correct PR number.
   - Keyboard: press Enter on focused card; assert `onSelectPR` called.
3. **DesignDeltaViewer tests**:
   - Mock diff API to return 2 files. Render; assert both file sections with filenames. Assert diff content visible.
   - Toggle inline/split view; assert the mode changes.
   - Click close; assert `onClose` called.
   - Mock diff API to return 500; assert error message.
4. **Accessibility audit tests**:
   - For each major state (list loaded, list empty, list error, diff viewer open), render and run `axe(container)`. Assert `results.violations` has length 0 for critical and serious.
5. **Integration test**: Render the full page with both API calls mocked. Click a card, verify diff viewer opens. Close it, verify list reappears and focus returns.
6. All async assertions should use `waitFor` or `findBy` queries.

## Validation
All tests must pass in CI. Verify coverage: DesignSnapshotList (loading, populated, empty, error, click, keyboard), DesignDeltaViewer (loaded, error, toggle, close), axe-core audit per state. Integration test covers the full user flow from list to diff and back.