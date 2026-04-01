Implement subtask 4002: Build DesignSnapshotList component with PR cards, loading skeletons, and error state

## Objective
Implement the `DesignSnapshotList` component that receives PR data (or loading/error states) and renders a list of PR cards with title, number, GitHub link, status badge, files changed count, and relative timestamp. Include loading skeleton placeholders and an error message UI.

## Steps
1. Create `components/DesignSnapshotList.tsx`. Accept props: `{ prs, isLoading, error, onSelectPR }`.
2. **Loading state**: When `isLoading` is true, render 3 skeleton cards using Tailwind's `animate-pulse` on placeholder divs matching card dimensions.
3. **Error state**: When `error` is truthy, render an error banner with the message and a 'Retry' button (calls a retry callback prop).
4. **Empty state**: When `prs` is an empty array, render a centered message: 'No design snapshots found for this run.'
5. **Populated state**: Map over `prs` and render a card for each:
   - PR title as an `<a>` linking to `pr.url` (opens in new tab with `rel='noopener noreferrer'`).
   - PR number displayed as `#${pr.number}`.
   - Status badge: green for 'open', purple for 'merged', red for 'closed'. Use a `<span>` with appropriate Tailwind bg/text classes.
   - Files changed: e.g., '5 files changed'.
   - Relative timestamp using a lightweight formatter (e.g., `Intl.RelativeTimeFormat` or a small utility function — no heavy dependency).
   - The entire card is clickable (calls `onSelectPR(pr.number)`) to trigger the diff viewer.
6. Cards should use a CSS grid layout: single column on mobile (<640px), 2 columns on tablet, 3 columns on desktop.

## Validation
Component test: render with 3 mocked PRs, assert 3 cards exist with correct titles, status badge colors, and GitHub links. Render with empty array, assert 'No design snapshots' message. Render with isLoading=true, assert skeleton elements. Render with error, assert error message and retry button.