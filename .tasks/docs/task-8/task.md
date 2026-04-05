## Develop Web Frontend (Blaze - React/Next.js)

### Objective
Build the Next.js 15 web frontend with equipment catalog, quote builder, Morgan web chat, and portfolio, using Effect for data fetching and validation.

### Ownership
- Agent: blaze
- Stack: React/Next.js
- Priority: high
- Status: pending
- Dependencies: 2, 7

### Implementation Details
{"steps":["Initialize Next.js 15 project with App Router, React 19, shadcn/ui, TailwindCSS 4, and Effect 3.x.","Implement pages: / (hero), /equipment (catalog), /equipment/:id (details), /quote (builder), /portfolio (gallery), /llms.txt, /llms-full.","Integrate with Equipment Catalog API for product data and availability.","Implement self-service quote builder with Effect form validation.","Embed Morgan web chat widget, connecting to Morgan agent.","Display portfolio synced from Social Media Engine.","Add Schema.org structured data and llms.txt for AI optimization.","Deploy to Cloudflare Pages."]}

### Subtasks
- [ ] Initialize Next.js 15 project with App Router, React 19, shadcn/ui, TailwindCSS 4, and Effect 3.x: Scaffold the Next.js 15 project with all core dependencies, configure the App Router layout structure, global styles, and shared components (header, footer, navigation).
- [ ] Implement hero landing page (/): Build the home page with a hero section, value proposition, featured equipment, and call-to-action sections.
- [ ] Implement equipment catalog listing page (/equipment) with API integration: Build the equipment catalog listing page that fetches and displays equipment from the Equipment Catalog API, with search, filtering, and pagination.
- [ ] Implement equipment detail page (/equipment/:id) with availability check: Build the individual equipment detail page showing full specs, images, availability, and a CTA to request a quote.
- [ ] Implement self-service quote builder page (/quote) with Effect form validation: Build the quote builder page allowing users to select equipment, specify rental details, and submit a quote request, using Effect Schema for form validation.
- [ ] Implement Morgan web chat widget integration: Build and embed the Morgan web chat widget that connects to the Morgan agent's WebSocket endpoint, supporting real-time conversational interaction.
- [ ] Implement portfolio gallery page (/portfolio) synced from Social Media Engine: Build the portfolio gallery page that displays published content (project photos, videos, case studies) synced from the Social Media Engine backend.
- [ ] Add Schema.org structured data and llms.txt pages for SEO and AI optimization: Implement Schema.org JSON-LD structured data across relevant pages and create /llms.txt and /llms-full routes for AI crawler optimization.
- [ ] Configure Cloudflare Pages deployment and end-to-end testing: Set up Cloudflare Pages deployment for the Next.js 15 application, configure environment variables, custom domain, and run end-to-end tests against the deployed site.