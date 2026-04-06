Implement subtask 8008: Add Schema.org structured data and llms.txt endpoints

## Objective
Implement Schema.org JSON-LD structured data across all pages and create the /llms.txt and /llms-full.txt endpoints for AI crawler optimization.

## Steps
1. Audit and consolidate Schema.org structured data across all pages:
   - Organization schema on home page
   - Product schema on catalog and detail pages
   - LocalBusiness schema in root layout
   - BreadcrumbList on all interior pages
2. Create `app/llms.txt/route.ts` — a Route Handler that returns a plain text file describing the site's content structure, purpose, and key pages for LLM crawlers.
3. Create `app/llms-full.txt/route.ts` (or `app/llms-full/route.ts`) — a more detailed version with full content descriptions, equipment categories, and service explanations.
4. Set appropriate `Content-Type: text/plain` headers.
5. Add `<link>` tags in the root layout pointing to llms.txt.
6. Validate all structured data with Google's Rich Results Test or Schema.org validator.

## Validation
/llms.txt returns a valid plain text response with site description; /llms-full.txt returns detailed content; Schema.org JSON-LD validates without errors on Google's Rich Results Test for home, catalog, and detail pages; BreadcrumbList renders correctly on interior pages.