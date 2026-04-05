Implement subtask 8002: Implement hero landing page (/)

## Objective
Build the hero landing page at the root route with the company value proposition, call-to-action sections, and responsive layout.

## Steps
1. Create `app/page.tsx` as the hero landing page.
2. Build a hero section with headline, subheadline, and primary CTA (e.g., 'Get a Quote', 'Browse Equipment').
3. Add secondary sections: featured equipment highlights, company overview, testimonials placeholder, and contact CTA.
4. Use shadcn/ui components for cards, buttons, and layout primitives.
5. Ensure fully responsive design (mobile, tablet, desktop).
6. Add Schema.org Organization structured data in the page's metadata.
7. Optimize for Core Web Vitals: lazy-load images, use Next.js Image component, minimize layout shift.

## Validation
Hero page renders at `/` with all sections visible; CTA buttons link to correct routes (/equipment, /quote); responsive layout works on mobile/tablet/desktop viewports; Schema.org JSON-LD is present in page source; no layout shift on initial load.