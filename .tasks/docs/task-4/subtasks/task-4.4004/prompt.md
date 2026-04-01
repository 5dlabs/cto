Implement subtask 4004: Accessibility pass: ARIA labels, keyboard navigation, and screen reader support

## Objective
Audit and enhance all components built in prior subtasks for accessibility compliance: add ARIA attributes, ensure full keyboard navigation, and verify screen reader compatibility for the PR card list and diff viewer.

## Steps
1. **PR cards (DesignSnapshotList)**:
   - Each card should be a `<button>` or have `role='button'` and `tabIndex={0}` with `onKeyDown` handling Enter/Space to trigger selection.
   - Add `aria-label` to each card: e.g., 'View diff for PR #42: Update design tokens, status open'.
   - Status badges should have `aria-label` describing the status (not rely on color alone).
   - The GitHub link inside the card must be separately focusable and have `aria-label='Open PR #42 on GitHub (opens in new tab)'`.
   - Loading skeletons should have `aria-busy='true'` and `aria-label='Loading design snapshots'`.
   - Error state retry button should have `aria-label='Retry loading design snapshots'`.
2. **DesignDeltaViewer**:
   - The view mode toggle should be a proper `<button>` group or radio group with `role='radiogroup'` and `aria-label='Diff view mode'`.
   - Collapsible file sections should use `<details>/<summary>` or `aria-expanded` on toggle buttons.
   - The diff viewer output should be wrapped in a `role='region'` with `aria-label='Diff for filename.ext'`.
   - Close button should have `aria-label='Close diff viewer'` and focus should return to the triggering card on close.
3. **Focus management**: When navigating from list to diff viewer, focus the first element in the diff viewer. When closing, return focus to the previously selected card.
4. Verify no color-only indicators — all status badges also have text labels.

## Validation
Run axe-core programmatically on the rendered page in each state (loading, populated, empty, error, diff view open). Assert zero critical or serious violations. Tab through all interactive elements and verify logical focus order. Verify focus moves to diff viewer on card activation and returns on close.