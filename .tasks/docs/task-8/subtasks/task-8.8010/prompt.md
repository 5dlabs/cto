Implement subtask 8010: Build SEO infrastructure: sitemap.xml, robots.txt, llms.txt, and llms-full routes

## Objective
Implement sitemap.xml generation, robots.txt, and the AI-native optimization routes `/llms.txt` and `/llms-full` returning plain text machine-readable site descriptions.

## Steps
1. Sitemap (`app/sitemap.ts`):
   - Export `generateSitemap()` function.
   - Include static routes: /, /equipment, /quote, /portfolio.
   - Dynamically include all product pages: fetch product IDs from catalog API, generate `/equipment/:id` entries.
   - Set appropriate `lastModified`, `changeFrequency`, `priority` for each.
2. Robots.txt (`app/robots.ts`):
   - Allow all user agents.
   - Reference sitemap URL.
   - Disallow `/dev/` (styleguide) if it exists.
3. `/llms.txt` route (`app/llms.txt/route.ts`):
   - Return `Response` with `Content-Type: text/plain`.
   - Content: site name, description, main sections, contact info, capabilities (equipment rental, lighting production, quote requests).
   - Follow emerging llms.txt convention.
4. `/llms-full` route (`app/llms-full/route.ts`):
   - Return `Response` with `Content-Type: text/plain`.
   - Fetch catalog summary from API: list all categories and product names with day rates.
   - Include services description, pricing ranges, location/service area.
   - More verbose than llms.txt for full AI agent consumption.
5. Add OpenGraph default image to `public/og-default.jpg`.
6. Create shared `lib/seo.ts` utility with default meta values, `generatePageMetadata` helper.

## Validation
Fetch `/sitemap.xml`, verify it returns valid XML with at least 5 URLs including dynamic product pages. Fetch `/robots.txt`, verify it allows all agents and references sitemap. Fetch `/llms.txt`, verify Content-Type is text/plain and body contains company name and service descriptions. Fetch `/llms-full`, verify it contains equipment catalog data. Verify `lib/seo.ts` helper produces correct metadata shape.