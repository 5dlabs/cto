Implement subtask 4004: Build Hermes deliberation dashboard page

## Objective
Create `app/hermes/page.tsx` — the deliberation list view with Card components showing deliberation details, status badges, pagination, and loading skeleton states.

## Steps
1. Create `app/hermes/page.tsx` as a client component (needs SWR hooks).
2. Use `useDeliberations(page, limit)` hook from the API client.
3. Render a grid/list of shadcn Card components, each showing:
   - Deliberation ID (truncated with copy-to-clipboard)
   - Status as a shadcn Badge with color coding: pending=gray, processing=blue, completed=green, failed=red
   - Triggered by (user/system identifier)
   - Timestamp (relative time, e.g., '5 minutes ago' with full date on hover)
   - Artifact count
   - Click navigates to `/hermes/[id]`
4. Pagination: read `page` from URL query params (`useSearchParams`), render previous/next buttons, update URL on navigation.
5. Loading state: render a grid of Skeleton components matching the Card layout while data is loading.
6. Empty state: when no deliberations exist, show an informative empty state message.
7. Error state: when the API call fails, show an error message with a retry button.
8. Page metadata: set title to 'Hermes Deliberations'.

## Validation
Component test: mock `useDeliberations` to return 3 deliberations, render the page, verify 3 Card elements are present, each containing a deliberation ID and a Badge. Verify Badge colors match status: completed→green variant, failed→red variant. Pagination test: mock 20 deliberations with limit 10, verify page 1 shows 10 cards and a 'Next' button. Loading test: mock loading state, verify Skeleton components render. Empty test: mock empty response, verify empty state message.