Implement task 8: Develop Web Frontend (Blaze - Next.js 15/React 19)

## Goal
Build the Sigma-1 website using Next.js 15 App Router, React 19, shadcn/ui, and TailwindCSS 4, with equipment catalog, self-service quote builder, Morgan web chat, and portfolio gallery.

## Task Context
- Agent owner: Blaze
- Stack: React/Next.js
- Priority: high
- Dependencies: 2, 7

## Implementation Plan
{"steps": ["Initialize Next.js 15 project with App Router, React 19, shadcn/ui, TailwindCSS 4, and Effect 3.x.", "Implement pages: / (hero, CTA), /equipment (catalog), /equipment/:id (details, availability), /quote (quote builder), /portfolio (gallery), /llms.txt, /llms-full.", "Integrate with Equipment Catalog API for product data and availability.", "Implement self-service quote builder with Effect form validation.", "Embed Morgan web chat widget.", "Display portfolio gallery with event photos and testimonials.", "Optimize for AI-native features: llms.txt, Schema.org structured data.", "Use TanStack Query + Effect for data fetching.", "Deploy to Cloudflare Pages.", "Ensure accessibility and responsive design."]}

## Acceptance Criteria
All pages render and load data from backend APIs; quote builder submits and receives confirmation; Morgan web chat is functional; portfolio gallery displays event photos; accessibility checks pass; site deploys to Cloudflare Pages and is reachable.

## Subtasks
- Initialize Next.js 15 project with App Router, React 19, shadcn/ui, TailwindCSS 4, Effect 3.x, and TanStack Query: Scaffold the Next.js 15 project with App Router, configure React 19, install and configure shadcn/ui component library, set up TailwindCSS 4, integrate Effect 3.x for typed error handling, and configure TanStack Query for data fetching with Effect integration.
- Implement equipment catalog pages with API integration: Build the /equipment listing page and /equipment/:id detail page, integrating with the Equipment Catalog API for product data, search, filtering, and real-time availability checking.
- Implement self-service quote builder with Effect form validation: Build the /quote page with a multi-step quote builder form that allows customers to select equipment, specify event details, and submit a quote request, using Effect 3.x for typed form validation and submission.
- Embed Morgan web chat widget: Integrate the Morgan AI web chat widget into the Sigma-1 website, connecting to the Morgan agent's WebSocket endpoint for real-time conversational UI with message history, typing indicators, and minimizable chat window.
- Implement portfolio gallery with event photos and testimonials: Build the /portfolio page displaying a gallery of past event photos and customer testimonials, with filtering by event type and a lightbox view for images.
- Implement home page with hero section and CTA: Build the / (home) page with a hero section, primary call-to-action, featured equipment highlights, trust signals, and quick links to key sections of the site.
- Implement AI-native features: llms.txt, llms-full, and Schema.org structured data: Create the /llms.txt and /llms-full routes serving machine-readable site descriptions for AI crawlers, and ensure comprehensive Schema.org structured data is present across all pages.
- Accessibility audit, responsive design polish, and Cloudflare Pages deployment: Conduct an accessibility audit (WCAG 2.1 AA) across all pages, polish responsive design for all breakpoints, and configure deployment to Cloudflare Pages with environment variables and build settings.

## Deliverables
- Update the relevant code, configuration, and tests.
- Keep artifacts aligned with the acceptance criteria.
- Document blockers or assumptions in your final summary.