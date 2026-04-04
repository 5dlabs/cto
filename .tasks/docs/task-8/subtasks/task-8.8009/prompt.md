Implement subtask 8009: Build Portfolio page with filterable gallery and lazy-loaded images

## Objective
Implement the `/portfolio` route as a statically generated gallery of past events with photos, filterable by event type, with lazy-loaded image grid and Schema.org Event markup.

## Steps
1. Create `app/portfolio/page.tsx` — Server Component with static generation (or ISR if content comes from API).
2. Data source: fetch from social engine published posts endpoint, or use static JSON/MDX content for v1.
3. Filter bar: horizontal pill buttons for event types (Concerts, Corporate, Festivals, Weddings, etc.). Clicking filters the grid client-side.
4. Image grid:
   - Masonry or uniform grid layout using CSS grid.
   - Each card: event image (lazy loaded via `<Image loading="lazy">`), event name overlay, event type badge, date.
   - Click opens Dialog/Sheet with larger image, full description, additional photos.
5. Lazy loading: only load images as they scroll into viewport (native lazy loading + Intersection Observer for animation).
6. SEO: `generateMetadata` with portfolio description. Schema.org: ItemList of Event entries.
7. Responsive: 1 col mobile, 2 cols tablet, 3 cols desktop.
8. Empty state: if no portfolio items match filter, show friendly message.

## Validation
Render portfolio page with mock data (5+ events across 3 types). Verify all events shown initially. Click a filter, verify only matching events shown. Click 'All', verify all events shown again. Verify images have `loading="lazy"` attribute. Open an event detail dialog, verify larger image and description shown. Test responsive layout at 375px, 768px, 1440px.