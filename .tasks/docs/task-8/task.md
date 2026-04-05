## Develop Sigma-1 Website (Blaze - Next.js/React/Effect)

### Objective
Build the Sigma-1 website using Next.js 15, React 19, shadcn/ui, TailwindCSS 4, and Effect. Includes equipment catalog, quote builder, portfolio, and Morgan web chat widget.

### Ownership
- Agent: blaze
- Stack: React/Next.js
- Priority: high
- Status: pending
- Dependencies: 2, 7, 6, 1

### Implementation Details
{"steps": ["Initialize Next.js 15 project with App Router, React 19, shadcn/ui, TailwindCSS 4, and Effect 3.x.", "Implement pages: / (hero), /equipment (catalog), /equipment/:id (product detail), /quote (quote builder), /portfolio (gallery), /llms.txt, /llms-full.", "Integrate with Equipment Catalog API for real-time data fetching using TanStack Query + Effect.", "Implement Effect Schema validation for all forms and API responses.", "Embed Morgan web chat widget and connect to agent endpoint.", "Implement portfolio sync with Social Media Engine for published content.", "Ensure accessibility (WCAG 2.1 AA), performance (LCP < 2s), and SEO (Schema.org, llms.txt).", "Write unit and integration tests for all pages and components."]}

### Subtasks
- [ ] Scaffold Next.js 15 project with App Router, React 19, TailwindCSS 4, shadcn/ui, Effect, and TanStack Query: Initialize the Next.js 15 project with all foundational dependencies configured: App Router, React 19, TailwindCSS 4, shadcn/ui component library, Effect 3.x, and TanStack Query for data fetching.
- [ ] Implement hero landing page (/): Build the homepage with hero section, value propositions, featured equipment, CTAs, and SEO metadata. This is the primary entry point for the Sigma-1 website.
- [ ] Implement equipment catalog listing page (/equipment): Build the equipment catalog listing page with search, filtering, sorting, and pagination, fetching data from the Equipment Catalog API.
- [ ] Implement equipment detail page (/equipment/:id): Build the individual equipment detail page showing full specifications, images, availability calendar, pricing, and a CTA to request a quote.
- [ ] Implement quote builder page (/quote) with form handling and API integration: Build the quote request page with a multi-step form for selecting equipment, specifying rental details, customer information, and submitting the quote request to the backend.
- [ ] Implement portfolio/gallery page (/portfolio) with Social Media Engine sync: Build the portfolio page that displays project showcases and published content synced from the Social Media Engine, with image gallery and filtering capabilities.
- [ ] Implement llms.txt and llms-full SEO pages: Create the /llms.txt and /llms-full routes that serve structured information about Sigma-1 optimized for LLM consumption, following the llms.txt specification.
- [ ] Embed Morgan web chat widget and connect to agent WebSocket endpoint: Implement the Morgan AI chat widget component that connects to the Morgan agent's web chat endpoint, providing real-time conversational AI support across all pages.
- [ ] Implement accessibility (WCAG 2.1 AA), performance optimization (LCP < 2s), and Schema.org SEO: Audit and enhance the entire site for WCAG 2.1 AA accessibility compliance, optimize performance to achieve LCP under 2 seconds, and verify Schema.org structured data across all pages.
- [ ] Write unit and integration tests for all pages and components: Develop comprehensive unit tests for individual components and integration tests for page-level data fetching, form submission, and widget interactions, targeting 80%+ code coverage.