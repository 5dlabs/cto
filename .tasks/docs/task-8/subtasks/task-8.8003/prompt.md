Implement subtask 8003: Implement equipment catalog listing page (/equipment) with API integration

## Objective
Build the equipment catalog listing page that fetches equipment data from the Equipment Catalog API using TanStack Query and displays it in a filterable, paginated view.

## Steps
1. Create `app/equipment/page.tsx` for the catalog listing.
2. Define the Equipment Catalog API client: base URL from environment variables, typed fetch functions using Effect.Schema for response validation.
3. Implement a TanStack Query hook (`useEquipmentList`) that fetches paginated equipment data from the API.
4. Build the equipment listing UI: grid/list view of equipment cards showing image, name, category, daily/weekly rate, and availability status.
5. Implement client-side filtering (by category, availability) and search.
6. Implement pagination or infinite scroll.
7. Add loading skeletons and error states.
8. Add Schema.org Product structured data for each equipment item.

## Validation
Equipment listing page renders at `/equipment`; data is fetched from the Equipment Catalog API and displayed; filtering by category works; pagination loads additional items; loading and error states render correctly; Effect.Schema validates API responses; Schema.org Product data is present.