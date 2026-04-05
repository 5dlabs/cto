Implement subtask 8003: Implement equipment catalog listing page (/equipment)

## Objective
Build the equipment catalog listing page with search, filtering, sorting, and pagination, fetching data from the Equipment Catalog API.

## Steps
1. Create app/equipment/page.tsx.
2. Implement equipment grid/list view with cards showing image, name, category, daily rate, availability status.
3. Implement search bar with debounced text search.
4. Implement filter controls: category, availability date range, price range, specifications.
5. Implement sorting: price (asc/desc), name, popularity.
6. Implement pagination or infinite scroll.
7. Use TanStack Query + Effect for data fetching with proper loading, error, and empty states.
8. Implement Effect Schema validation for API response data.
9. Add URL-based filter state (searchParams) so filters are shareable/bookmarkable.
10. Add Schema.org Product structured data for equipment items.

## Validation
Catalog page renders equipment items from the API; search filters results correctly; category and price filters work; sorting changes item order; pagination loads additional items; URL params reflect current filters; loading and error states display correctly; Schema.org Product data is present.