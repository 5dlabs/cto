Implement subtask 8003: Implement equipment catalog page with API integration

## Objective
Build the equipment catalog listing page at `/equipment` that fetches equipment data from the Equipment Catalog API, displays filterable/searchable results, and links to individual product detail pages.

## Steps
1. Create `app/equipment/page.tsx` for the catalog listing.
2. Implement server-side data fetching (RSC) from the Equipment Catalog API search endpoint using the Effect-based API client.
3. Build an EquipmentCard component displaying: image, name, category, daily rate, availability indicator.
4. Implement search input with debounced client-side filtering or server-side search param.
5. Implement category filter sidebar/dropdown.
6. Handle loading states (skeleton cards) and error states (API unavailable).
7. Implement pagination or infinite scroll for large catalogs.
8. Add Schema.org Product structured data for each listed item.
9. Ensure images use Next.js `<Image>` component with proper sizing and lazy loading.

## Validation
Catalog page renders at `/equipment`; equipment items are fetched from the API and displayed as cards; search filters results correctly; category filter narrows listings; loading skeleton appears during fetch; error state renders when API is mocked as unavailable; pagination works; each card links to `/equipment/:id`.