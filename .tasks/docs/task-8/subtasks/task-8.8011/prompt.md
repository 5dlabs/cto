Implement subtask 8011: Implement /llms.txt and /llms-full routes for AI agent consumption

## Objective
Create the /llms.txt static text file route describing Sigma-1 services, equipment categories, and pricing model for AI agent consumption, and the /llms-full route with a complete machine-readable catalog dump.

## Steps
1. Create `app/llms.txt/route.ts` as a Next.js Route Handler:
   - Return `text/plain` content type.
   - Content: structured text describing Sigma-1 services, all 24 equipment categories with brief descriptions, pricing model (day rates), contact info, how to get a quote, Morgan AI assistant availability.
   - Format: follow the llms.txt convention (title, description, sections with headers).
2. Create `app/llms-full/route.ts` as a Next.js Route Handler:
   - Return `application/json` or `text/plain` (machine-readable).
   - Content: full equipment catalog dump (fetch from Equipment Catalog API at request time or build time), including all product names, categories, day rates, specifications.
   - Include service descriptions, FAQs, and contact methods.
3. Add appropriate Cache-Control headers: `public, max-age=3600` (1 hour) for both routes.
4. Link to /llms.txt from the footer and robots.txt.

## Validation
Test: GET /llms.txt returns 200 with Content-Type text/plain, body contains 'Sigma-1', lists equipment categories, and includes pricing information. GET /llms-full returns 200 with machine-readable content containing full catalog data. Both responses include Cache-Control headers.