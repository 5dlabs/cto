Implement subtask 8004: Build equipment catalog listing page (/equipment) with filters and data table

## Objective
Implement the /equipment route with a filterable, sortable equipment catalog listing that fetches data from the Equipment Catalog API using the shared data fetching layer.

## Steps
1. Create `app/equipment/page.tsx` as a server component shell with metadata.
2. Create `app/equipment/equipment-catalog.tsx` client component that uses `useEquipmentList()` hook.
3. Implement filter controls: category select, search input, availability date range picker (use a date picker from shadcn/ui or a lightweight library).
4. Use the DataTable component from 8002 to render equipment rows with columns: image thumbnail, name, category, daily rate, availability status.
5. Implement URL-based filter state using `useSearchParams` so filters are bookmarkable/shareable.
6. Add Skeleton loading states while data fetches.
7. Implement pagination using TanStack Query's `keepPreviousData` for smooth page transitions.
8. Add empty state and error state components.
9. Make the listing responsive: card grid on mobile, table on desktop.

## Validation
Page renders at /equipment with mock API data. Filters update URL search params and re-fetch filtered data. Sorting changes column order. Pagination navigates between pages. Skeleton states appear during loading. Empty state renders when no results match filters. Responsive layout switches between card and table views at breakpoints.