Implement subtask 8007: Implement portfolio gallery page (/portfolio) with social engine sync

## Objective
Build the portfolio gallery page that displays published social media content fetched from the social engine backend, showcasing project photos and event content.

## Steps
1. Create `app/portfolio/page.tsx` for the portfolio gallery.
2. Define the social content API client: fetch published social posts from the social engine backend endpoint.
3. Implement a TanStack Query hook (`usePortfolioContent`) to fetch and cache published content.
4. Build the gallery UI: masonry or grid layout displaying images/videos with captions, dates, and platform badges.
5. Implement filtering by content type or tag if the API supports it.
6. Add lightbox or modal view for full-size image/video viewing.
7. Implement loading skeletons and empty state ('No content yet').
8. Add Schema.org ImageGallery structured data.

## Validation
Portfolio page renders at `/portfolio`; content is fetched from the social engine API; gallery displays images with captions; lightbox opens for full-size viewing; filtering works if supported; loading and empty states render correctly; Schema.org structured data is present.