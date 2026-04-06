Implement subtask 8004: Implement equipment product detail page

## Objective
Build the individual equipment detail page at `/equipment/:id` that fetches and displays full product information, availability calendar, specifications, and a CTA to add the item to a quote.

## Steps
1. Create `app/equipment/[id]/page.tsx` as a dynamic route.
2. Fetch equipment details from the Equipment Catalog API by ID using server components.
3. Display: hero image/gallery, name, description, category, daily/weekly/monthly rates.
4. Display specifications in a structured table or list.
5. Show availability status (available, rented, maintenance) with visual indicator.
6. Add an 'Add to Quote' CTA button that navigates to /quote with the equipment ID pre-selected.
7. Implement `generateMetadata` for dynamic SEO titles and descriptions.
8. Add Schema.org Product structured data with pricing, availability.
9. Handle 404 case when equipment ID doesn't exist (use `notFound()`).

## Validation
Detail page renders at `/equipment/valid-id` with correct product data; image, specs, and pricing display correctly; 'Add to Quote' button links to /quote with equipment context; `/equipment/invalid-id` returns 404; Schema.org Product JSON-LD is present with correct data; page metadata includes product name.