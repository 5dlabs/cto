Implement subtask 8008: Add Schema.org structured data and llms.txt pages for SEO and AI optimization

## Objective
Implement Schema.org JSON-LD structured data across relevant pages and create /llms.txt and /llms-full routes for AI crawler optimization.

## Steps
1. Add Schema.org JSON-LD structured data:
   - Home page: Organization, LocalBusiness schemas.
   - Equipment listing: ItemList schema.
   - Equipment detail: Product schema with offers, availability.
   - Portfolio: CreativeWork or ImageGallery schema.
2. Create `app/llms.txt/route.ts` as a Next.js route handler returning plain text with site summary, key pages, and navigation hints for LLM crawlers.
3. Create `app/llms-full/route.ts` returning a more comprehensive plain text representation of the site's content and capabilities.
4. Add meta tags (title, description, og:image) to all pages via generateMetadata.
5. Create a sitemap.xml via `app/sitemap.ts`.
6. Verify structured data with Google's Rich Results Test or Schema.org validator.

## Validation
View page source on each page; JSON-LD script tags contain correct Schema.org data. /llms.txt returns plain text with site description. /llms-full returns comprehensive content. Google Rich Results Test validates structured data without errors. sitemap.xml lists all public routes.