Implement subtask 8002: Implement hero landing page (/)

## Objective
Build the homepage with hero section, value propositions, featured equipment, CTAs, and SEO metadata. This is the primary entry point for the Sigma-1 website.

## Steps
1. Create app/(marketing)/page.tsx as the landing page.
2. Implement the hero section with headline, subheadline, and primary CTA (e.g., 'Browse Equipment', 'Get a Quote').
3. Add a featured equipment section that fetches top items from the Equipment Catalog API using TanStack Query + Effect.
4. Add value proposition cards (reliability, fleet size, service areas).
5. Add a testimonials/social proof section.
6. Implement responsive design for mobile, tablet, desktop.
7. Add Schema.org Organization structured data.
8. Configure page metadata (title, description, Open Graph tags).
9. Ensure all interactive elements are keyboard accessible.

## Validation
Homepage renders with all sections visible; featured equipment loads from API; CTA links navigate correctly; Schema.org JSON-LD is present in page source; page metadata is correct; responsive layout works at 320px, 768px, and 1280px breakpoints.