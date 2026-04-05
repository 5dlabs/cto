Implement subtask 8008: Implement SEO optimization: Schema.org structured data, llms.txt, and llms-full pages

## Objective
Add comprehensive Schema.org structured data across all pages, create the /llms.txt and /llms-full routes for AI discoverability, and optimize for Lighthouse score >90.

## Steps
1. Audit and consolidate Schema.org structured data across all pages: Organization on /, Product on /equipment and /equipment/:id, ItemList on /equipment, LocalBusiness if applicable.
2. Create `app/llms.txt/route.ts` that returns a plain text file describing the site's purpose, capabilities, and API endpoints for AI consumption.
3. Create `app/llms-full/route.ts` (or page) that returns a comprehensive machine-readable description of all equipment, services, and interaction capabilities.
4. Add comprehensive meta tags, Open Graph tags, and Twitter Card tags to all pages via Next.js Metadata API.
5. Implement sitemap.xml and robots.txt via Next.js conventions.
6. Run Lighthouse audits and optimize: ensure images use next/image, minimize JavaScript bundles, add proper heading hierarchy, ensure all interactive elements are accessible.
7. Fix any accessibility issues (ARIA labels, keyboard navigation, color contrast).
8. Verify Lighthouse Performance, Accessibility, Best Practices, and SEO scores are all >90.

## Validation
Schema.org JSON-LD is present and valid on all pages (validate with Google Rich Results Test); /llms.txt returns valid plain text content; /llms-full returns comprehensive site description; sitemap.xml and robots.txt are accessible; Lighthouse scores >90 in all four categories; Open Graph tags render correctly in social media preview tools.