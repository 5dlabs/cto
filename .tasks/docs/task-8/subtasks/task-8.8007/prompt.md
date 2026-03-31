Implement subtask 8007: Build homepage (/), portfolio gallery (/portfolio), and static content pages

## Objective
Implement the homepage with hero section and CTAs, the portfolio gallery page, and any other static content pages needed for the site.

## Steps
1. Create `app/page.tsx` (homepage): hero section with headline, subheadline, and primary CTA (link to /quote or /equipment), secondary CTA. Featured equipment section (can use static data or fetch top items). Services overview section. Testimonials or trust signals section. Final CTA banner.
2. Create `app/portfolio/page.tsx`: gallery grid of past projects/events. Each item: image, title, description, equipment used. Use a responsive masonry or grid layout. Optional: lightbox for full-size images. Can be statically generated from local data or CMS.
3. Ensure all pages use consistent layout (navigation, footer from 8002).
4. Add appropriate metadata for each page (title, description, og tags).
5. Create `app/not-found.tsx` global 404 page with helpful navigation links.
6. Create `app/loading.tsx` global loading state.

## Validation
Homepage renders with hero, featured equipment, services, and CTA sections. All CTAs link to correct destinations. Portfolio page renders gallery grid with images. 404 page renders for unknown routes. All pages have correct metadata in document head. Pages are responsive across mobile, tablet, and desktop breakpoints.