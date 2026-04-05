Implement subtask 8007: Implement llms.txt and llms-full SEO pages

## Objective
Create the /llms.txt and /llms-full routes that serve structured information about Sigma-1 optimized for LLM consumption, following the llms.txt specification.

## Steps
1. Create app/llms.txt/route.ts as a route handler returning plain text.
2. Define the llms.txt content: company name, description, services, equipment categories, contact info, and links to key pages.
3. Create app/llms-full/route.ts (or page.tsx if HTML) with expanded information including full equipment catalog summary, service descriptions, pricing tiers, and operational details.
4. Ensure content is kept in sync with actual business data (can be statically defined for v1).
5. Set correct Content-Type headers (text/plain for llms.txt).
6. Add link to llms.txt in the site's HTML head and robots.txt.

## Validation
/llms.txt returns valid plain text with correct Content-Type header; /llms-full returns expanded content; both routes are accessible and contain accurate Sigma-1 business information; llms.txt link is present in HTML head.