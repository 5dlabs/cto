Implement subtask 8002: Implement home page and equipment catalog pages (/, /equipment, /equipment/:id)

## Objective
Build the landing page, equipment listing page with filtering/search, and individual equipment detail page, all integrated with the Equipment Catalog API.

## Steps
1. Build the home page (/) with hero section, featured equipment, value propositions, and CTAs linking to /equipment and /quote.
2. Build the /equipment listing page: fetch equipment data from the Catalog API using TanStack Query + Effect for error handling.
3. Implement filtering (by category, availability, price range) and search functionality on the listing page.
4. Implement pagination or infinite scroll for the equipment list.
5. Build the /equipment/:id detail page: fetch individual equipment details, display images, specifications, pricing, and availability calendar.
6. Add an 'Add to Quote' or 'Request Quote' CTA on the detail page that links to /quote with pre-filled equipment.
7. Create the API client module in /lib that wraps Catalog API calls with Effect for typed error handling and TanStack Query for caching.
8. Handle loading states, error states, and empty states for all data-fetching scenarios.

## Validation
Home page renders with featured equipment from API; /equipment lists equipment with correct data; filters narrow results correctly; /equipment/:id shows correct details for a given ID; 'Add to Quote' navigates to /quote with equipment context; loading and error states display correctly.