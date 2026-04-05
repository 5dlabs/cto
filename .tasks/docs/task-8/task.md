## Develop Sigma-1 Website (Blaze - Next.js 15/React 19)

### Objective
Build the Sigma-1 website with equipment catalog, self-service quote builder, Morgan web chat, and portfolio gallery, optimized for AI-native features.

### Ownership
- Agent: Blaze
- Stack: Next.js 15/React 19/Effect
- Priority: high
- Status: pending
- Dependencies: 2, 6, 7

### Implementation Details
{"steps": ["Initialize Next.js 15 project with App Router, React 19, shadcn/ui, TailwindCSS 4, and Effect 3.x.", "Implement pages: / (hero), /equipment (catalog), /equipment/:id (details), /quote (builder), /portfolio (gallery), /llms.txt, /llms-full.", "Integrate with Equipment Catalog API for real-time data.", "Implement Effect.Schema for validation and TanStack Query for data fetching.", "Embed Morgan web chat widget.", "Display published social content in portfolio.", "Add Schema.org structured data and llms.txt for AI optimization.", "Deploy to Cloudflare Pages."]}

### Subtasks
- [ ] Initialize Next.js 15 project with App Router, React 19, shadcn/ui, TailwindCSS 4, Effect 3.x, and Cloudflare Pages deployment: Bootstrap the Next.js 15 project with all required dependencies, configure the App Router, set up the design system foundation (shadcn/ui + TailwindCSS 4), integrate Effect 3.x, configure TanStack Query, and establish the Cloudflare Pages deployment pipeline.
- [ ] Implement hero landing page (/): Build the hero landing page at the root route with the company value proposition, call-to-action sections, and responsive layout.
- [ ] Implement equipment catalog listing page (/equipment) with API integration: Build the equipment catalog listing page that fetches equipment data from the Equipment Catalog API using TanStack Query and displays it in a filterable, paginated view.
- [ ] Implement equipment detail page (/equipment/:id) with API integration: Build the individual equipment detail page that fetches a single equipment item from the API and displays full details, specifications, availability calendar, and a CTA to add to quote.
- [ ] Implement self-service quote builder page (/quote) with Effect.Schema validation: Build the interactive quote builder page where customers can select equipment, specify rental dates, enter contact information, and submit a quote request with full Effect.Schema validation.
- [ ] Embed Morgan web chat widget on the website: Integrate the Morgan AI web chat widget into the website, connecting it to the Morgan agent's WebSocket/HTTP chat endpoint.
- [ ] Implement portfolio gallery page (/portfolio) with social engine sync: Build the portfolio gallery page that displays published social media content fetched from the social engine backend, showcasing project photos and event content.
- [ ] Implement SEO optimization: Schema.org structured data, llms.txt, and llms-full pages: Add comprehensive Schema.org structured data across all pages, create the /llms.txt and /llms-full routes for AI discoverability, and optimize for Lighthouse score >90.