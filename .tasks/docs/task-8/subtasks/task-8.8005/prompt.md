Implement subtask 8005: Build Equipment Catalog page with TanStack Table v8, filtering, search, and pagination

## Objective
Implement the `/equipment` catalog page using TanStack Table v8 with category filtering (24 categories), debounced text search, price range filter, pagination (20 items/page), grid/list view toggle, and column visibility controls. Data fetching via Effect + TanStack Query.

## Steps
1. Create `app/equipment/page.tsx` as a Client Component (interactive table).
2. Define the TanStack Table v8 instance in `components/sigma1/equipment-table.tsx`:
   - Columns: image thumbnail, name, category (Badge), day rate (formatted currency), availability indicator.
   - Column visibility toggle dropdown.
3. Filter sidebar or top-bar:
   - Category filter: dropdown/multi-select with all 24 equipment categories. Selecting a category adds it as a query param and refetches.
   - Text search: Input with 300ms debounce (use a custom `useDebouncedValue` hook). Calls the Equipment Catalog API search endpoint.
   - Price range filter: min/max inputs or a simple range selector.
4. Pagination:
   - Server-side pagination, 20 items per page.
   - TanStack Table pagination controls: previous, next, page number display, total count.
5. Grid/list view toggle:
   - List view: traditional table rows.
   - Grid view: responsive card grid (3 columns desktop, 2 tablet, 1 mobile).
   - Toggle state stored in URL search params or local component state.
6. Data fetching layer in `lib/api/equipment.ts`:
   - Define Effect services/layers for equipment API calls.
   - Wrap in TanStack Query hooks: `useEquipmentList({ page, category, search, priceMin, priceMax })`.
   - Stale time: 1 minute.
7. Loading state: show Skeleton components while data loads.
8. Empty state: "No equipment found" message with suggestion to clear filters.
9. Each product row/card links to `/equipment/[id]`.

## Validation
Component test: render equipment table with mock data of 40 items, verify 20 items shown on first page, clicking 'next' shows remaining 20. Test category filter: selecting a category triggers API call with category param. Test search: typing in search input waits 300ms then triggers API call. Test grid/list toggle: switching view changes rendered markup from table to card grid. Verify skeleton loading state appears before data resolves.