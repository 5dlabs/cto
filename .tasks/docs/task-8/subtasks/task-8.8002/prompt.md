Implement subtask 8002: Implement home page with hero section and CTA

## Objective
Build the marketing home page at `/` with a hero section, value propositions, featured equipment highlights, and call-to-action buttons directing users to the catalog and quote builder.

## Steps
1. Create `app/(marketing)/page.tsx` as the home page.
2. Build a Hero component with headline, subheadline, and primary CTA button (link to /equipment and /quote).
3. Add a featured equipment section that displays 3-4 highlighted items (can be static for v1 or fetched from catalog API).
4. Add a value proposition section with icons/cards explaining the rental service.
5. Add a CTA banner section encouraging visitors to get a quote.
6. Ensure responsive design: mobile-first with breakpoints for tablet and desktop.
7. Add Schema.org Organization structured data in the page head.
8. Implement proper semantic HTML (h1, sections, nav landmarks).

## Validation
Home page renders at `/`; hero section displays with correct text and CTA links; featured equipment section renders cards; responsive layout works at mobile (375px), tablet (768px), and desktop (1280px) widths; Schema.org JSON-LD is present in page source; Lighthouse accessibility score >90.