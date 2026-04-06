Implement subtask 8005: Build portfolio page (/portfolio) synced from Social Media Engine

## Objective
Implement the portfolio page that displays published project content fetched from the Social Media Engine API.

## Steps
1. Create the /portfolio page with a grid/masonry layout for showcasing project photos and content.
2. Fetch published portfolio items from the Social Media Engine API using TanStack Query + Effect.
3. Display each portfolio item with image(s), title, description, date, and any relevant tags/categories.
4. Implement filtering by category or project type if the API supports it.
5. Add lightbox or modal view for full-size image viewing.
6. Handle loading, empty, and error states.
7. Ensure images are optimized using Next.js Image component with proper sizing and lazy loading.

## Validation
Portfolio page renders published items from the Social Media Engine API; images display correctly with lazy loading; filtering works if implemented; lightbox opens full-size images; empty state displays when no items exist; loading state shows during fetch.