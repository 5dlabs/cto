Implement subtask 8004: Implement equipment detail page (/equipment/:id) with availability check

## Objective
Build the individual equipment detail page showing full specs, images, availability, and a CTA to request a quote.

## Steps
1. Create `app/equipment/[id]/page.tsx` as a dynamic route.
2. Fetch equipment details by ID from the Equipment Catalog API using Effect.
3. Display: equipment name, description, full-size images (gallery/carousel), specifications table, daily/weekly/monthly rates.
4. Integrate availability check: call sigma1_check_availability or the catalog availability endpoint, display available dates or availability status.
5. Add a 'Request Quote' CTA button that links to /quote with the equipment pre-selected (via query params or state).
6. Add related/similar equipment section at the bottom.
7. Implement metadata for SEO (title, description, og:image).
8. Handle 404 for non-existent equipment IDs.

## Validation
Page renders at /equipment/[valid-id] with correct equipment data. Image gallery is interactive. Availability status is displayed. 'Request Quote' CTA links to /quote with equipment context. /equipment/[invalid-id] shows 404 page. SEO meta tags are present in page source.