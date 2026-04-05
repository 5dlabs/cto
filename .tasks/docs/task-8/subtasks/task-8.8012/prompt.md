Implement subtask 8012: Implement SEO infrastructure: Schema.org structured data, sitemap, Open Graph, and metadata

## Objective
Add Schema.org JSON-LD structured data to all pages (Organization on home, Product on equipment detail, Event on portfolio), generate sitemap.xml, configure Open Graph meta tags, and set up the Next.js metadata API for per-page SEO.

## Steps
1. Create a shared `lib/seo/structured-data.ts` with helper functions to generate JSON-LD objects:
   - `organizationSchema()`: Organization type with Sigma-1 details.
   - `productSchema(product)`: Product type with name, image, description, offers (day rate).
   - `eventSchema(portfolioItem)`: Event type for portfolio items.
2. Add JSON-LD script tags to respective pages using a `<JsonLd data={...} />` component.
3. Create `app/sitemap.ts` using Next.js sitemap generation:
   - Static routes: /, /equipment, /quote, /portfolio.
   - Dynamic routes: /equipment/[id] for all products (fetch product IDs from API at build time).
   - Set changefreq and priority appropriately.
4. Create `app/robots.ts` with Next.js robots generation: allow all, reference sitemap URL, reference /llms.txt.
5. Verify each page has proper `metadata` export with unique title, description, and Open Graph image.
6. Add canonical URLs to prevent duplicate content issues.

## Validation
Test: view page source of / and verify Organization JSON-LD is present and valid. View source of /equipment/[id] and verify Product JSON-LD with correct product data. GET /sitemap.xml returns valid XML with all static routes and at least one dynamic /equipment/[id] route. GET /robots.txt references sitemap and allows all crawlers. Validate structured data using Google's Rich Results Test schema validator.