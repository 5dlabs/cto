Implement subtask 8006: Implement portfolio/gallery page (/portfolio) with Social Media Engine sync

## Objective
Build the portfolio page that displays project showcases and published content synced from the Social Media Engine, with image gallery and filtering capabilities.

## Steps
1. Create app/portfolio/page.tsx.
2. Fetch published portfolio items from the Social Media Engine API using TanStack Query + Effect.
3. Implement a masonry or grid gallery layout for project images/videos.
4. Add filtering by project type, equipment used, or date.
5. Implement a detail modal or expandable card for individual portfolio items showing full description, equipment used, and social media links.
6. Implement Effect Schema validation for the Social Media Engine API response.
7. Add lazy loading for images with blur placeholders.
8. Add Schema.org ImageGallery or CreativeWork structured data.
9. Ensure images have alt text for accessibility.

## Validation
Portfolio page renders items from the Social Media Engine API; gallery layout displays correctly on all screen sizes; filtering reduces displayed items appropriately; detail view shows complete information; images lazy-load with placeholders; alt text is present on all images.