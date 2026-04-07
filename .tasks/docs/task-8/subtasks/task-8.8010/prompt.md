Implement subtask 8010: Build portfolio page (app/portfolio/page.tsx) with masonry grid and lightbox

## Objective
Implement the event portfolio page fetching published social posts and displaying them in a masonry grid with a lightbox modal for full-size image viewing.

## Steps
1. Create app/portfolio/page.tsx as a Server Component. Fetch published posts: runApiEffect(getPublishedPosts()).
2. Pass posts to client component components/portfolio/PortfolioGrid.tsx.
3. Masonry grid: use CSS columns (column-count: 2 on mobile, 3 on tablet, 4 on desktop) with break-inside-avoid on each card. Or use a lightweight masonry library if CSS columns have ordering issues.
4. Portfolio card: next/image with fill and object-fit: cover, caption overlay on hover (semi-transparent dark gradient), hashtags as small badges.
5. Lightbox: on card click, open a Dialog (shadcn) containing the full-size image, caption, hashtags, and published date. Close button and keyboard Escape support.
6. Handle empty state: if no published posts, show message 'Portfolio coming soon' with placeholder grid of Skeleton cards.
7. Add 'Projects' text to nav item in layout if not already labeled 'Portfolio'.

## Validation
Playwright: GET /portfolio renders image grid with >= 1 card (using mocked API via MSW returning 3 posts). Click first card — Dialog opens with full-size image and caption. Press Escape — Dialog closes. On 375px viewport — grid renders in 2 columns without overflow.