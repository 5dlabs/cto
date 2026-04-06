## Develop Website Frontend (Blaze - Next.js 15/React 19/Effect)

### Objective
Build the Sigma-1 website with equipment catalog, self-service quote builder, portfolio, and embedded Morgan web chat, optimized for AI and SEO.

### Ownership
- Agent: blaze
- Stack: Next.js 15/React 19/Effect
- Priority: high
- Status: pending
- Dependencies: 2, 7

### Implementation Details
{"steps": ["Initialize Next.js 15 project with App Router, React 19, shadcn/ui, TailwindCSS 4, and Effect 3.x", "Implement pages: /, /equipment, /equipment/:id, /quote, /portfolio, /llms.txt, /llms-full", "Integrate with Equipment Catalog API for product and availability data using TanStack Query + Effect", "Build self-service quote builder with Effect form validation", "Embed Morgan web chat widget and ensure real-time communication", "Display portfolio synced from Social Media Engine", "Add Schema.org structured data and llms.txt for AI optimization", "Ensure accessibility, responsive design, and performance best practices"]}

### Subtasks
- [ ] Initialize Next.js 15 project with App Router, React 19, TailwindCSS 4, shadcn/ui, Effect 3.x, and TanStack Query: Scaffold the Next.js 15 project with all core dependencies, configure the design system foundation, and set up the API data-fetching layer.
- [ ] Implement home page and equipment catalog pages (/, /equipment, /equipment/:id): Build the landing page, equipment listing page with filtering/search, and individual equipment detail page, all integrated with the Equipment Catalog API.
- [ ] Build self-service quote builder page (/quote) with Effect form validation: Implement the quote builder allowing customers to select equipment, specify rental dates and location, and submit a quote request with full form validation using Effect.
- [ ] Embed Morgan web chat widget with real-time WebSocket communication: Integrate the Morgan AI chat widget into the website as a floating component that communicates with the Morgan agent's WebSocket endpoint in real-time.
- [ ] Build portfolio page (/portfolio) synced from Social Media Engine: Implement the portfolio page that displays published project content fetched from the Social Media Engine API.
- [ ] Add Schema.org structured data, llms.txt routes, and SEO metadata: Implement Schema.org JSON-LD structured data across all pages, create /llms.txt and /llms-full routes for AI optimization, and configure comprehensive SEO metadata.
- [ ] Accessibility audit, responsive design QA, and Lighthouse performance optimization: Ensure the entire site passes accessibility checks, is fully responsive across devices, and achieves a Lighthouse score above 90 in all categories.