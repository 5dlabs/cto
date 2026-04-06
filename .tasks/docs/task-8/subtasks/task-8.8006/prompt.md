Implement subtask 8006: Implement portfolio gallery page with Social Media Engine integration

## Objective
Build the portfolio page at `/portfolio` that fetches published content (project photos, case studies) from the Social Media Engine API and displays them in an attractive gallery layout.

## Steps
1. Create `app/portfolio/page.tsx`.
2. Fetch portfolio/published content from the Social Media Engine API using RSC.
3. Build a responsive gallery grid layout (masonry or standard grid).
4. Each gallery item shows: image, project title, brief description, date.
5. Implement a lightbox or modal for full-size image viewing.
6. Add category/tag filtering if the API provides categorized content.
7. Handle empty state (no portfolio items yet) with a friendly message.
8. Handle API errors gracefully.
9. Use Next.js `<Image>` with proper optimization for gallery images.

## Validation
Portfolio page renders at `/portfolio`; gallery items are fetched from Social Media Engine API and displayed; responsive grid layout works at all breakpoints; lightbox opens on item click; empty state renders when no items are returned; API error state shows fallback UI; images load with proper optimization.