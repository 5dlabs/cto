Implement subtask 8004: Implement equipment detail page (/equipment/:id)

## Objective
Build the individual equipment detail page showing full specifications, images, availability calendar, pricing, and a CTA to request a quote.

## Steps
1. Create app/equipment/[id]/page.tsx with dynamic routing.
2. Fetch equipment details from the Equipment Catalog API using the item ID.
3. Display: image gallery/carousel, equipment name, description, full specifications table.
4. Display pricing information (daily, weekly, monthly rates).
5. Show availability calendar or date picker for checking availability.
6. Add 'Request Quote' CTA that links to /quote with equipment pre-selected.
7. Add related/similar equipment section.
8. Implement Effect Schema validation for the detail API response.
9. Add comprehensive Schema.org Product structured data (name, image, offers, availability).
10. Implement generateMetadata for dynamic SEO (title, description, OG image).

## Validation
Detail page renders correct equipment data for a given ID; image gallery navigates between images; specifications table displays all fields; availability check returns results; 'Request Quote' CTA links to /quote with correct equipment context; 404 page shows for invalid IDs; Schema.org Product data is complete and valid.