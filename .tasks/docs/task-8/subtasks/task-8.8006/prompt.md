Implement subtask 8006: Add Schema.org structured data, llms.txt routes, and SEO metadata

## Objective
Implement Schema.org JSON-LD structured data across all pages, create /llms.txt and /llms-full routes for AI optimization, and configure comprehensive SEO metadata.

## Steps
1. Add Schema.org JSON-LD structured data to the home page (LocalBusiness, Organization).
2. Add Schema.org Product structured data to /equipment/:id pages (name, description, image, price, availability).
3. Add Schema.org structured data to /portfolio items (CreativeWork or ImageObject).
4. Create /llms.txt route (app/llms.txt/route.ts) that returns a plain-text summary of the site's purpose, capabilities, and API endpoints for AI crawlers.
5. Create /llms-full route with comprehensive structured content about all equipment categories, services, and business information.
6. Configure Next.js metadata API in each page's layout/page.tsx: title, description, Open Graph tags, Twitter cards, canonical URLs.
7. Add robots.txt and sitemap.xml generation (using next-sitemap or custom route handlers).
8. Verify all structured data passes Google's Rich Results Test validator.

## Validation
Schema.org JSON-LD is present in page source for home, equipment detail, and portfolio pages; /llms.txt returns valid plain-text content; /llms-full returns comprehensive site info; Open Graph tags render correctly in social sharing preview tools; Google Rich Results Test validates structured data without errors; sitemap.xml lists all public routes.