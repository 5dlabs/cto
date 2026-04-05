Implement task 8: Develop Web Frontend (Blaze - React/Next.js)

## Goal
Build the Sigma-1 website with Next.js 15 App Router, React 19, shadcn/ui, TailwindCSS 4, and Effect for data fetching/validation. Features equipment catalog with real-time availability, self-service quote builder, Morgan web chat widget, project portfolio, and AI-native optimization (llms.txt, Schema.org).

## Task Context
- Agent owner: blaze
- Stack: React 19/Next.js 15 + Effect 3.x
- Priority: high
- Dependencies: 2, 7

## Implementation Plan
1. Initialize Next.js 15 project with App Router, TypeScript 5.x, TailwindCSS 4, shadcn/ui.
   - Configure Effect 3.x integration for data fetching and schema validation.
   - Set up TanStack Query with Effect for server state management.
   - Establish design tokens referencing existing sigma-1.com visual direction: dark/moody palette appropriate for lighting/visual production company, professional typography, generous spacing.
2. Pages and routes:
   - **`/` (Home)**: Hero section with video/image background showcasing lighting production, value proposition, CTA buttons ("Browse Equipment", "Get a Quote", "Chat with Morgan"). SEO metadata, Schema.org Organization markup.
   - **`/equipment` (Catalog)**: Server-rendered product grid with category sidebar. Filterable by category, searchable by name. Pagination. Each product card shows image (from R2 CDN), name, day rate, availability indicator. Uses `GET /api/v1/catalog/categories` and `GET /api/v1/catalog/products`.
   - **`/equipment/:id` (Product Detail)**: Full product info with image gallery, specs table (from JSONB), day rate, availability calendar component. Date range picker triggers `GET /api/v1/catalog/products/:id/availability`. "Add to Quote" button. Effect Schema validation for date inputs.
   - **`/quote` (Quote Builder)**: Multi-step form: 1) Select products (from catalog, with availability check), 2) Event details (date range, venue, contact info), 3) Review & submit. Form state managed with Effect. Submits to Morgan or directly to RMS opportunities endpoint. Confirmation page with opportunity reference.
   - **`/portfolio` (Portfolio)**: Gallery of past events with photos (from social engine published posts or static content). Filterable by event type. Lazy-loaded image grid.
   - **`/llms.txt` (AI)**: Static route returning plain text machine-readable site description for AI agents.
   - **`/llms-full` (AI)**: Full content dump including equipment catalog summary, services, pricing ranges.
3. Morgan Web Chat Widget:
   - Persistent bottom-right floating button (per D10 resolution).
   - On click, expands to near-full-screen chat interface.
   - WebSocket connection to Morgan agent.
   - Message history persisted in localStorage with session ID.
   - Support for rich messages: product cards, quote summaries, availability results.
   - Typing indicators, message status (sent/delivered/read).
   - Minimizable, closable, remembers state across page navigations.
4. Component library:
   - Install and configure shadcn/ui components: Button, Card, Dialog, Form, Input, Select, Table, Tabs, Badge, Calendar, Sheet, Popover.
   - Custom components: ProductCard, AvailabilityCalendar, QuoteLineItem, ChatBubble, ChatWidget.
   - Design tokens in `tailwind.config.ts`: colors (dark theme with accent colors), border radius, font family.
5. Data fetching pattern:
   - Server Components for initial data (categories, product lists) using Effect + fetch.
   - Client Components with TanStack Query + Effect for interactive data (availability checks, search).
   - API base URL from environment variable pointing to backend services (or API gateway).
6. SEO and AI optimization:
   - Schema.org structured data on all pages: Organization, Product (on equipment pages), Event (on portfolio).
   - OpenGraph and Twitter Card meta tags.
   - Sitemap.xml generation.
   - `robots.txt` allowing all crawlers.
7. Accessibility:
   - shadcn/ui provides Radix UI accessibility primitives.
   - Verify WCAG 2.1 AA compliance: keyboard navigation, screen reader labels, color contrast.
8. Performance:
   - Image optimization via Next.js Image component with R2 CDN loader.
   - Static generation for home, portfolio pages.
   - Dynamic rendering for equipment catalog (filterable/searchable).
   - Target: Lighthouse Performance > 90, Accessibility > 95.
9. Deployment:
   - Configure for Cloudflare Pages deployment (using `@cloudflare/next-on-pages` adapter).
   - Environment variables for API endpoints.
   - Build output optimized for edge runtime where possible.

## Acceptance Criteria
1. Component unit tests (Vitest + React Testing Library): ProductCard renders name, image, price; AvailabilityCalendar shows available/unavailable dates; ChatWidget opens/closes correctly. 2. Page integration tests: `/equipment` page renders product grid after mocking catalog API; `/equipment/:id` shows product details and availability calendar. 3. Quote builder flow test: add 2 products, fill event details, submit, verify API call made with correct payload. 4. Chat widget test: mock WebSocket, send message, verify message appears in chat, verify typing indicator shows during response. 5. SEO test: verify Schema.org JSON-LD present on home page (Organization), equipment page (Product). Verify `llms.txt` returns plain text with correct content. 6. Accessibility audit: run axe-core on all pages, verify zero critical/serious violations. 7. Lighthouse CI: Performance > 90, Accessibility > 95, Best Practices > 90 on home and equipment pages. 8. Cloudflare Pages build: `npx @cloudflare/next-on-pages` completes without errors. 9. Responsive test: verify equipment grid renders correctly at mobile (375px), tablet (768px), desktop (1440px) widths.

## Subtasks
- Initialize Next.js 15 project with App Router, TypeScript, and TailwindCSS 4: Scaffold the Next.js 15 project with App Router, TypeScript 5.x strict mode, and TailwindCSS 4. Configure project structure with app directory, layout files, global styles, and environment variable handling for API endpoints and CDN URLs.
- Configure shadcn/ui component library and design token system: Install and configure shadcn/ui with Radix UI primitives. Define the dark/moody design token system in tailwind.config.ts reflecting the Sigma-1 lighting/production company brand: dark palette, accent colors, professional typography, generous spacing.
- Set up Effect 3.x integration and TanStack Query data fetching layer: Configure Effect 3.x for schema validation and data fetching. Integrate TanStack Query with Effect programs for client-side server state management. Create reusable API client utilities and Effect schemas for catalog, availability, and quote payloads.
- Build Home page with hero section, CTAs, and Schema.org Organization markup: Implement the `/` (Home) route as a statically generated page with a video/image hero background showcasing lighting production, value proposition text, CTA buttons ('Browse Equipment', 'Get a Quote', 'Chat with Morgan'), and Schema.org Organization JSON-LD.
- Build Equipment Catalog listing page with filtering, search, and pagination: Implement the `/equipment` route with a server-rendered product grid, category sidebar filter, search-by-name input, and pagination. Each ProductCard displays image, name, day rate, and availability indicator. Data fetched from catalog API endpoints.
- Build Product Detail page with image gallery, specs table, and Add to Quote: Implement the `/equipment/:id` route showing full product information: image gallery, specifications table (from JSONB data), day rate, and an 'Add to Quote' button that stores selection in quote builder state.
- Build AvailabilityCalendar component with date range picker: Create the AvailabilityCalendar custom component that displays available/unavailable dates for a product and integrates a date range picker. Fetches availability data from the API with Effect Schema validation for date inputs.
- Build Quote Builder multi-step form: Implement the `/quote` route with a multi-step form: Step 1 — select products with availability check, Step 2 — event details (date range, venue, contact info), Step 3 — review and submit. Form state managed with Effect, submission to backend API.
- Build Portfolio page with filterable gallery and lazy-loaded images: Implement the `/portfolio` route as a statically generated gallery of past events with photos, filterable by event type, with lazy-loaded image grid and Schema.org Event markup.
- Build SEO infrastructure: sitemap.xml, robots.txt, llms.txt, and llms-full routes: Implement sitemap.xml generation, robots.txt, and the AI-native optimization routes `/llms.txt` and `/llms-full` returning plain text machine-readable site descriptions.
- Build Morgan web chat widget with WebSocket connection and rich messages: Implement the persistent floating chat widget for Morgan AI agent: bottom-right floating button, expandable near-full-screen chat UI, WebSocket connection, rich message support (product cards, quote summaries), typing indicators, localStorage message persistence, and minimizable state.
- Accessibility audit and WCAG 2.1 AA compliance verification: Run comprehensive accessibility audit across all pages using axe-core, verify WCAG 2.1 AA compliance including keyboard navigation, screen reader labels, color contrast, and focus management. Fix any violations found.
- Performance optimization and Lighthouse CI targets: Optimize all pages for Lighthouse Performance > 90 and Accessibility > 95. Configure Next.js Image with R2 CDN loader, static generation for appropriate pages, code splitting, and bundle analysis.
- Configure Cloudflare Pages deployment with @cloudflare/next-on-pages: Set up the project for Cloudflare Pages deployment using the @cloudflare/next-on-pages adapter. Configure edge runtime compatibility, environment variables, build output, and wrangler configuration.

## Deliverables
- Update the relevant code, configuration, and tests.
- Keep artifacts aligned with the acceptance criteria.
- Document blockers or assumptions in your final summary.