Implement subtask 8009: Add Schema.org structured data and llms.txt endpoints for AI optimization

## Objective
Implement Schema.org JSON-LD structured data across all pages and create the /llms.txt and /llms-full text endpoints for AI discoverability.

## Steps
1. Create `@/lib/structured-data.ts` with helper functions to generate JSON-LD objects for: Organization (global), Product (equipment detail pages), ItemList (equipment catalog), WebSite (homepage), BreadcrumbList (all pages).
2. Add JSON-LD `<script>` tags to the appropriate pages via `<head>` metadata or a StructuredData component.
3. Homepage: Organization + WebSite schema.
4. /equipment: ItemList schema with equipment items.
5. /equipment/[id]: Product schema with name, description, image, offers (rental rates).
6. Create `app/llms.txt/route.ts` (Next.js Route Handler) that returns a plain text response describing the site's purpose, available pages, and API endpoints in a format optimized for LLM consumption.
7. Create `app/llms-full/route.ts` that returns a more detailed plain text version with full content descriptions, equipment categories, and service details.
8. Set appropriate `Content-Type: text/plain` headers and cache headers on llms.txt routes.

## Validation
Validate JSON-LD on homepage, /equipment, and /equipment/[id] using Google Rich Results Test or Schema.org validator. /llms.txt returns plain text with correct Content-Type header. /llms-full returns extended content. Structured data includes correct Organization name, Product details, and BreadcrumbList paths.