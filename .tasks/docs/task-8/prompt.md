Implement task 8: Build Website Frontend (Blaze - Next.js 15/React 19/Effect)

## Goal
Build the Sigma-1 public website with AI-optimized pages, equipment catalog with real-time availability, self-service quote builder, project portfolio gallery, and persistent Morgan web chat widget. Uses Next.js 15 App Router, React 19, shadcn/ui with custom Sigma-1 branding, TailwindCSS 4, and Effect for data fetching and validation.

## Task Context
- Agent owner: blaze
- Stack: Next.js 15/React 19/Effect
- Priority: high
- Dependencies: 2, 7

## Implementation Plan
1. Initialize Next.js 15 project with App Router:
   - `npx create-next-app@15` with TypeScript, TailwindCSS 4, ESLint, App Router
   - Install: shadcn/ui (latest), Effect 3.x, @tanstack/react-query, @tanstack/react-table v8, react-hook-form v7, @hookform/resolvers
   - Configure biome.js for linting alongside ESLint
2. Design system setup — reference https://deployiq.maximinimal.ca for brand baseline:
   - Define Sigma-1 brand tokens in `tailwind.config.ts`:
     - Primary colors (professional dark theme suitable for lighting/production company)
     - Typography: modern sans-serif (Inter or similar)
     - Spacing, border radius, shadow tokens
   - Create `components/ui/` via shadcn/ui init, customize theme CSS variables
   - Create custom component files: `components/sigma1/quote-builder.tsx`, `components/sigma1/chat-widget.tsx`, `components/sigma1/availability-calendar.tsx`
3. Page implementations:
   - **`/` (Home)**: Hero section with video/image background showcasing lighting setups, value proposition ("One platform, one conversation"), CTA buttons (Browse Equipment, Get a Quote, Chat with Morgan), featured equipment carousel, testimonial section. Server-rendered static content.
   - **`/equipment` (Catalog)**: TanStack Table v8 displaying 533+ products with:
     - Column visibility toggles
     - Category filter (sidebar or dropdown with 24 categories)
     - Text search (debounced, calls Equipment Catalog API)
     - Price range filter
     - Pagination (20 items per page)
     - Grid/list view toggle
     - Effect-based data fetching via TanStack Query + Effect adapter
   - **`/equipment/:id` (Product Detail)**: Product images (R2 CDN), specifications from JSONB specs, day rate, availability calendar component showing available/booked dates for next 90 days, "Add to Quote" button. Effect Schema validation on API response.
   - **`/quote` (Quote Builder)**: Multi-step form using React Hook Form v7:
     - Step 1: Event details (date range picker, venue, event type)
     - Step 2: Equipment selection (search/browse, add items with quantity, useFieldArray for dynamic line items)
     - Step 3: Review (line items, pricing, availability conflicts highlighted)
     - Step 4: Contact info + submit
     - Real-time price calculation as items added
     - Availability validation on date selection (calls availability API per item)
     - Effect Schema integration for form validation
   - **`/portfolio` (Portfolio)**: Masonry grid of published social media content, filterable by event type. Data from Social Engine published posts API.
   - **`/llms.txt`**: Static text file describing Sigma-1 services, equipment categories, pricing model for AI agent consumption
   - **`/llms-full`**: Complete machine-readable dump of catalog and services
4. Morgan Web Chat Widget (`components/sigma1/chat-widget.tsx`):
   - Persistent Intercom-style widget fixed to bottom-right corner
   - Survives page navigation (state lifted to layout level)
   - WebSocket connection to Morgan's /ws/chat endpoint
   - Session management: generate session token on first open, store in localStorage, send on reconnect
   - Server-side session backed by Valkey (handled by Morgan)
   - UI: message bubbles, typing indicator, streaming text display (character-by-character), minimize/maximize toggle
   - Accessible: ARIA labels, keyboard navigation, screen reader support
5. SEO and AI optimization:
   - Schema.org structured data on all pages (Organization, Product, Event)
   - Open Graph meta tags
   - Next.js metadata API for per-page title, description
   - Sitemap.xml generation
6. TanStack Query configuration:
   - Custom query client with Effect integration layer
   - Stale time: 1 min for catalog, 5 min for categories, no cache for availability
   - Error boundary integration
7. Responsive design: mobile-first, breakpoints for tablet and desktop.
8. Cloudflare Pages deployment configuration:
   - `wrangler.toml` or Cloudflare Pages project settings
   - Environment variables for API base URLs
   - Edge caching configuration for static pages
9. Accessibility: WCAG 2.1 AA compliance target, test with axe-core.

## Acceptance Criteria
1. Component test: Quote builder renders all 4 steps, adding 3 items via useFieldArray produces correct line items array and total price calculation. 2. Component test: Equipment catalog table renders 20 items, category filter reduces results, search input triggers debounced API call. 3. Component test: Chat widget opens, sends message via WebSocket mock, displays streamed response character by character. 4. Integration test: /equipment page fetches from Equipment Catalog API, renders product cards with images, prices in correct format. 5. E2E test (Playwright): navigate Home → /equipment → click product → /equipment/:id → click Add to Quote → /quote → fill form → submit, verify API call made with correct payload. 6. Accessibility test: axe-core scan on /, /equipment, /quote pages reports zero critical/serious violations. 7. Lighthouse test: / page scores >= 90 on Performance, Accessibility, SEO. 8. Session continuity test: open chat widget, send message, navigate to /equipment, verify chat history persists. 9. llms.txt test: GET /llms.txt returns text/plain with service description and equipment category listing.

## Subtasks
- Initialize Next.js 15 project with TypeScript, TailwindCSS 4, and core dependencies: Scaffold the Next.js 15 App Router project, install all required dependencies (shadcn/ui, Effect 3.x, TanStack Query, TanStack Table v8, react-hook-form v7, @hookform/resolvers), and configure biome.js alongside ESLint for linting.
- Set up Sigma-1 design system with TailwindCSS 4 brand tokens and shadcn/ui customization: Define the Sigma-1 brand design tokens in tailwind.config.ts (colors, typography, spacing, shadows for a professional dark theme), initialize shadcn/ui with customized CSS variables, and create the foundational UI component set.
- Build root layout with navigation, footer, responsive shell, and TanStack Query provider: Implement the root App Router layout with site-wide navigation header, footer, responsive mobile menu, TanStack Query client provider with Effect integration layer, and error boundary setup.
- Implement Home page with hero section, featured equipment carousel, and CTAs: Build the `/` home page with hero section (video/image background showcasing lighting setups), value proposition messaging, CTA buttons (Browse Equipment, Get a Quote, Chat with Morgan), featured equipment carousel, and testimonial section. Server-rendered static content.
- Build Equipment Catalog page with TanStack Table v8, filtering, search, and pagination: Implement the `/equipment` catalog page using TanStack Table v8 with category filtering (24 categories), debounced text search, price range filter, pagination (20 items/page), grid/list view toggle, and column visibility controls. Data fetching via Effect + TanStack Query.
- Build Product Detail page with specifications and Add to Quote functionality: Implement the `/equipment/[id]` dynamic product detail page showing product images (from R2 CDN), JSONB specifications, day rate pricing, and an 'Add to Quote' button that adds the item to the quote builder's state. Include Effect Schema validation on API responses.
- Build Availability Calendar component with 90-day booking view: Create a reusable availability calendar component that displays available/booked dates for the next 90 days for a given equipment item. Fetches availability data from the API and visually distinguishes available, booked, and partially-available dates.
- Build Quote Builder multi-step form with React Hook Form, dynamic line items, and real-time pricing: Implement the `/quote` page with a 4-step form using React Hook Form v7: Step 1 (event details with date range picker), Step 2 (equipment selection with useFieldArray for dynamic line items), Step 3 (review with pricing and availability conflict highlighting), Step 4 (contact info and submission). Includes real-time price calculation and Effect Schema validation.
- Build persistent Morgan Web Chat Widget with WebSocket, streaming text, and session management: Implement the Intercom-style persistent chat widget fixed to bottom-right corner that connects to Morgan's /ws/chat endpoint via WebSocket, supports streaming text display (character-by-character), manages sessions via localStorage, and survives page navigation by lifting state to the root layout.
- Build Portfolio page with masonry grid and event type filtering: Implement the `/portfolio` page displaying a masonry grid of published social media content from the Social Engine API, with filtering by event type.
- Implement /llms.txt and /llms-full routes for AI agent consumption: Create the /llms.txt static text file route describing Sigma-1 services, equipment categories, and pricing model for AI agent consumption, and the /llms-full route with a complete machine-readable catalog dump.
- Implement SEO infrastructure: Schema.org structured data, sitemap, Open Graph, and metadata: Add Schema.org JSON-LD structured data to all pages (Organization on home, Product on equipment detail, Event on portfolio), generate sitemap.xml, configure Open Graph meta tags, and set up the Next.js metadata API for per-page SEO.
- Configure Cloudflare Pages deployment with environment variables and edge caching: Set up the Cloudflare Pages deployment configuration including wrangler.toml or Cloudflare Pages project settings, environment variable configuration for API base URLs, and edge caching settings for static pages.
- Implement WCAG 2.1 AA accessibility compliance and axe-core testing setup: Audit and fix all pages and components for WCAG 2.1 AA compliance, set up axe-core automated accessibility testing, and configure Playwright tests for accessibility validation on key pages.
- Write E2E test suite with Playwright for critical user flows and Lighthouse performance validation: Create comprehensive Playwright E2E tests covering the primary user journey (Home → Equipment → Product → Quote → Submit) and Lighthouse performance audits ensuring the home page scores >= 90 on Performance, Accessibility, and SEO.

## Deliverables
- Update the relevant code, configuration, and tests.
- Keep artifacts aligned with the acceptance criteria.
- Document blockers or assumptions in your final summary.