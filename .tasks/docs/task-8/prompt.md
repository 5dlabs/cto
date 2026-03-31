Implement task 8: Develop Web Frontend (Blaze - React/Next.js 15 + Effect)

## Goal
Build the Sigma-1 website with equipment catalog, self-service quote builder, Morgan web chat, and portfolio gallery. Implements AI-native optimizations and integrates with backend APIs.

## Task Context
- Agent owner: blaze
- Stack: React 19, Next.js 15, Effect, shadcn/ui, TailwindCSS 4
- Priority: high
- Dependencies: 2, 7

## Implementation Plan
{"steps": ["Initialize Next.js 15 project with App Router, React 19, shadcn/ui, TailwindCSS 4, and Effect 3.x.", "Implement pages: / (hero, CTA), /equipment (catalog), /equipment/:id (details, availability), /quote (quote builder), /portfolio (gallery), /llms.txt, /llms-full.", "Integrate TanStack Query + Effect for data fetching from Equipment Catalog and Morgan APIs.", "Embed Morgan web chat widget.", "Implement Effect.Schema validation for forms and API responses.", "Add Schema.org structured data and llms.txt for AI optimization.", "Deploy to Cloudflare Pages."]}

## Acceptance Criteria
All pages render and fetch data from live APIs. Quote builder submits and receives confirmation. Morgan web chat is interactive. Lighthouse score >90. llms.txt and structured data are present.

## Subtasks
- Scaffold Next.js 15 project with App Router, React 19, TailwindCSS 4, and Effect 3.x: Initialize the Next.js 15 project using App Router, configure React 19, install and configure TailwindCSS 4, and set up Effect 3.x with the project's base TypeScript configuration. Establish the root layout, global styles, font loading, and base metadata.
- Set up shadcn/ui component library and shared UI primitives: Install and configure shadcn/ui, initialize the component registry, and add the base set of shared UI primitives (Button, Card, Input, Dialog, Sheet, Badge, Skeleton, Separator, NavigationMenu) that will be used across all pages. Resolve dp-12 and dp-13.
- Set up TanStack Query + Effect data fetching layer and API client services: Create the shared data fetching infrastructure: TanStack Query provider, Effect-based API client services for the Equipment Catalog API and Morgan API, and Effect.Schema definitions for all API response types.
- Build equipment catalog listing page (/equipment) with filters and data table: Implement the /equipment route with a filterable, sortable equipment catalog listing that fetches data from the Equipment Catalog API using the shared data fetching layer.
- Build equipment detail page (/equipment/:id) with availability checker: Implement the /equipment/[id] dynamic route showing full equipment details, image gallery, specifications, and an interactive availability checker that queries the backend.
- Build quote builder page (/quote) with multi-step form and Effect.Schema validation: Implement the self-service quote builder as a multi-step form with equipment selection, rental details, contact information, and review/submit steps, all validated with Effect.Schema.
- Build homepage (/), portfolio gallery (/portfolio), and static content pages: Implement the homepage with hero section and CTAs, the portfolio gallery page, and any other static content pages needed for the site.
- Embed Morgan web chat widget: Integrate the Morgan AI chat widget into the site as a floating chat interface available on all pages, connecting to the Morgan chat API.
- Add Schema.org structured data and llms.txt endpoints for AI optimization: Implement Schema.org JSON-LD structured data across all pages and create the /llms.txt and /llms-full text endpoints for AI discoverability.
- Configure Cloudflare Pages deployment and Lighthouse optimization: Set up Cloudflare Pages deployment configuration, optimize the application for Lighthouse performance score >90, and ensure production build is correct.

## Deliverables
- Update the relevant code, configuration, and tests.
- Keep artifacts aligned with the acceptance criteria.
- Document blockers or assumptions in your final summary.