Implement subtask 8010: Build Portfolio page with masonry grid and event type filtering

## Objective
Implement the `/portfolio` page displaying a masonry grid of published social media content from the Social Engine API, with filtering by event type.

## Steps
1. Create `app/portfolio/page.tsx`.
2. Data fetching: use TanStack Query + Effect to fetch published posts from Social Engine API. Server-side initial fetch for SEO, client-side hydration for interactivity.
3. Filter bar: horizontal pill/chip filters for event types (All, Concert, Corporate, Wedding, Festival, etc.). Clicking a filter fetches filtered data.
4. Masonry grid layout:
   - Use CSS `columns` property or CSS Grid with `masonry` behavior (fallback for browsers without native masonry: use a simple multi-column CSS approach).
   - Each card: image/video thumbnail, event name, date, short description.
   - Click to open a modal/lightbox with full content, larger image, and link to original social post.
5. Responsive: 3 columns desktop, 2 tablet, 1 mobile.
6. Loading: skeleton grid while data fetches.
7. SEO: metadata export with title/description, Open Graph image (could be first portfolio item).

## Validation
Component test: render portfolio with 10 mock posts, verify masonry grid renders all 10 items. Test filter: click 'Concert' filter, verify API called with event type param, grid re-renders with filtered results. Test responsive: at 375px width, verify single column layout. Test modal: click a portfolio item, verify modal opens with full content.