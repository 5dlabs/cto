Implement subtask 8003: Implement equipment catalog listing page (/equipment) with API integration

## Objective
Build the equipment catalog listing page that fetches and displays equipment from the Equipment Catalog API, with search, filtering, and pagination.

## Steps
1. Create `app/equipment/page.tsx` for the catalog listing.
2. Implement an Effect-based API client service for the Equipment Catalog API (search, list, filter endpoints).
3. Use Effect for data fetching with proper error handling (loading, error, empty states).
4. Display equipment as a grid of cards (image, name, category, daily rate, availability badge).
5. Implement search bar with debounced input.
6. Implement category filter (dropdown or sidebar with equipment categories).
7. Implement pagination or infinite scroll.
8. Add loading skeletons during data fetch.
9. Handle API errors gracefully with user-friendly messages.

## Validation
Page renders at /equipment. Equipment cards display with correct data from API. Search filters results in real-time. Category filter narrows results. Pagination works. Loading state shows skeletons. API error displays friendly message.