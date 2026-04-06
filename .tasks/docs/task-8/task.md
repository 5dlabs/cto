## Develop Website Frontend (Blaze - React/Next.js)

### Objective
Build the Sigma-1 website with equipment catalog, self-service quote builder, portfolio, and Morgan web chat. Use Next.js 15 App Router, shadcn/ui, TailwindCSS 4, and Effect for type safety.

### Ownership
- Agent: Blaze
- Stack: Next.js 15/React 19/Effect
- Priority: high
- Status: pending
- Dependencies: 2, 7

### Implementation Details
{"steps": ["Initialize Next.js 15 project with App Router, React 19, shadcn/ui, TailwindCSS 4, and Effect 3.x.", "Implement pages: / (hero, CTA), /equipment (catalog), /equipment/:id (product detail), /quote (quote builder), /portfolio (gallery), /llms.txt, /llms-full.", "Integrate with Equipment Catalog API for real-time data.", "Implement self-service quote builder with Effect form validation.", "Embed Morgan web chat widget.", "Fetch portfolio data from Social Media Engine.", "Add Schema.org structured data and llms.txt for AI optimization.", "Deploy to Cloudflare Pages."]}

### Subtasks
- [ ] Scaffold Next.js 15 project with App Router, React 19, shadcn/ui, TailwindCSS 4, and Effect: Initialize the Next.js 15 project with all foundational dependencies, configure App Router, set up TailwindCSS 4, install and configure shadcn/ui components, integrate Effect 3.x, and establish the project's folder structure and shared layout.
- [ ] Implement home page with hero section and CTA: Build the marketing home page at `/` with a hero section, value propositions, featured equipment highlights, and call-to-action buttons directing users to the catalog and quote builder.
- [ ] Implement equipment catalog page with API integration: Build the equipment catalog listing page at `/equipment` that fetches equipment data from the Equipment Catalog API, displays filterable/searchable results, and links to individual product detail pages.
- [ ] Implement equipment product detail page: Build the individual equipment detail page at `/equipment/:id` that fetches and displays full product information, availability calendar, specifications, and a CTA to add the item to a quote.
- [ ] Implement self-service quote builder with Effect form validation: Build the interactive quote builder page at `/quote` where users can select equipment, specify rental dates, enter delivery details, and submit a quote request. Use Effect Schema for robust form validation.
- [ ] Implement portfolio gallery page with Social Media Engine integration: Build the portfolio page at `/portfolio` that fetches published content (project photos, case studies) from the Social Media Engine API and displays them in an attractive gallery layout.
- [ ] Embed Morgan web chat widget: Integrate the Morgan AI web chat widget into the website as a persistent floating component that connects to Morgan's WebSocket endpoint for real-time customer conversations.
- [ ] Add Schema.org structured data and llms.txt endpoints: Implement Schema.org JSON-LD structured data across all pages and create the /llms.txt and /llms-full.txt endpoints for AI crawler optimization.
- [ ] Configure and deploy to Cloudflare Pages: Set up the Cloudflare Pages project, configure build settings for Next.js 15, set environment variables for backend API endpoints, and deploy the production build.