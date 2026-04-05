Implement subtask 8004: Implement equipment detail page (/equipment/:id) with API integration

## Objective
Build the individual equipment detail page that fetches a single equipment item from the API and displays full details, specifications, availability calendar, and a CTA to add to quote.

## Steps
1. Create `app/equipment/[id]/page.tsx` as a dynamic route.
2. Implement a TanStack Query hook (`useEquipmentDetail`) that fetches a single equipment item by ID.
3. Build the detail UI: large image gallery, equipment name, description, full specifications table, rental rates (daily/weekly/monthly), and real-time availability indicator.
4. Add a 'Add to Quote' or 'Request Quote' CTA button that links to the quote builder with this equipment pre-selected.
5. Implement loading and error states (including 404 for invalid IDs).
6. Add Schema.org Product structured data with full details.
7. Generate static params for popular items if beneficial for SEO (optional ISR).

## Validation
Detail page renders at `/equipment/[id]` with correct data for a valid ID; image gallery displays; specifications table is populated; availability indicator reflects API data; 'Add to Quote' button navigates to /quote with equipment context; 404 page renders for invalid IDs; Schema.org Product data is complete.