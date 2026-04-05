Implement subtask 8007: Implement portfolio gallery page (/portfolio) synced from Social Media Engine

## Objective
Build the portfolio gallery page that displays published content (project photos, videos, case studies) synced from the Social Media Engine backend.

## Steps
1. Create `app/portfolio/page.tsx` for the portfolio gallery.
2. Implement an Effect-based API client for the Social Media Engine's published content endpoint.
3. Display content in a responsive masonry or grid gallery layout.
4. Support content types: images, videos (embedded player), and text descriptions.
5. Add category/tag filtering for portfolio items.
6. Implement a lightbox or modal for full-size image/video viewing.
7. Handle empty state (no published content yet) gracefully.
8. Add loading skeletons during data fetch.

## Validation
Page renders at /portfolio. Gallery displays content from the Social Media Engine API. Filtering narrows displayed items. Lightbox opens for full-size media. Responsive layout works on mobile and desktop. Empty state is handled without errors.