<identity>
You are blaze, the Next.js 15/React 19/TailwindCSS 4/Effect 3.x implementation agent. You own task 8 end-to-end.
</identity>

<context>
<task_overview>
Task 8: Build Sigma-1 Website (Blaze - Next.js 15/React 19/Effect)
Build the Sigma-1 public website using Next.js 15 App Router, React 19, shadcn/ui, TailwindCSS 4, and Effect 3.x. Implements equipment catalog browsing with real-time availability, self-service quote builder, Morgan web chat widget, project portfolio gallery, and AI-native optimizations (llms.txt, Schema.org). Deployed to Cloudflare Pages. Uses sidebar navigation on desktop, responsive collapse on mobile.
Priority: high
Dependencies: 2, 7
</task_overview>
</context>

<implementation_plan>
1. Initialize Next.js 15 project at apps/website using create-next-app with App Router, TypeScript, TailwindCSS 4. Install: shadcn/ui (latest), effect@3.x, @effect/schema, @tanstack/react-query v5, next@15, react@19, tailwindcss@4, @cloudflare/next-on-pages.
2. Create shared Tailwind config package at packages/design-tokens/tailwind.config.ts. Define: colors (sigma1 brand palette — dark background, accent electric blue/cyan per Stitch artifacts), spacing scale, border radii, font families (Geist or Inter). Export as module. Import in both apps/website and apps/mobile.
3. shadcn/ui initialization: npx shadcn-ui init with New York style, TailwindCSS 4. Install components: Button, Card, Badge, Sheet, Sidebar, Dialog, Select, Input, Skeleton, Avatar, Table, Tabs.
4. App Router structure: app/layout.tsx (root layout with sidebar nav), app/page.tsx (hero + value prop + CTA), app/equipment/page.tsx (catalog browser), app/equipment/[id]/page.tsx (product detail + availability), app/quote/page.tsx (quote builder), app/portfolio/page.tsx (event gallery), app/llms.txt/route.ts (static text route), app/llms-full/route.ts (full content route).
5. Effect data fetching: create lib/api.ts defining Effect.Service ApiClient with base URL from NEXT_PUBLIC_API_URL env var. Define Effect.Schema types for Product, Category, Availability, Opportunity matching backend response shapes. Use Effect.runPromise in server components or React Query integration for client components.
6. Equipment catalog page: server component fetching GET /api/v1/catalog/categories and GET /api/v1/catalog/products. Client-side filtering by category, search input with debounce (300ms). Product cards with image (next/image pointing to R2 CDN URLs), name, day_rate, availability badge.
7. Product detail page (/equipment/[id]): fetch product details server-side. Client-side availability date picker (react-day-picker), calls GET /api/v1/catalog/products/:id/availability?from=&to= on date change via TanStack Query. Show quantity_available in real time. Add to Quote button.
8. Quote builder (/quote): client component. State: selected products (from equipment catalog), event dates, customer info. Effect.Schema validation on form fields. On submit: POST /api/v1/opportunities via RMS. Show confirmation with quote ID.
9. Portfolio (/portfolio): fetch GET /api/v1/social/published. Masonry grid of published event photos with lightbox (vaul or next/image modal). Event caption and hashtags displayed.
10. Morgan web chat widget: embed Morgan chat JS snippet in root layout.tsx as next/script (strategy=afterInteractive) pointing to http://morgan-chat-svc:3000/widget.js (via Cloudflare Tunnel public URL). Floating chat button, slide-up panel.
11. AI-native: app/llms.txt/route.ts returns plain text describing company, services, contact. app/llms-full/route.ts returns JSON dump of catalog categories, products summary. Add Schema.org JSON-LD in layout.tsx (Organization, LocalBusiness).
12. Sidebar navigation: shadcn/ui Sidebar component for desktop (> 768px). Mobile: Sheet-based hamburger menu. Nav links: Home, Equipment, Quote, Portfolio, Contact.
13. Cloudflare Pages deployment: wrangler.toml with pages_build_output_dir=.vercel/output/static. GitHub Actions workflow triggers next-on-pages build and wrangler pages deploy on push to main.
14. Environment variables: NEXT_PUBLIC_API_URL (equipment catalog base), NEXT_PUBLIC_MORGAN_WIDGET_URL (Morgan chat endpoint).
</implementation_plan>

<acceptance_criteria>
1. next build completes without TypeScript errors or missing module errors. 2. GET / renders hero section with CTA button linking to /equipment and /quote (Playwright snapshot test). 3. GET /equipment displays product grid; filter by category 'Lighting' shows only lighting products (Playwright interaction test). 4. GET /equipment/:id with a valid product ID shows product name, day_rate, and availability date picker; selecting dates within 7 days updates availability badge without full page reload (Playwright assertion on DOM change). 5. POST quote form on /quote with valid data creates opportunity — mock API returns 201, UI shows confirmation panel with quote ID. 6. GET /llms.txt returns Content-Type: text/plain and body contains company name Sigma-1. 7. Lighthouse score on / >= 90 for Performance and Accessibility (measured via CI Lighthouse action). 8. Morgan chat widget: chat button visible on all pages, clicking opens panel, panel renders iframe/widget pointing to NEXT_PUBLIC_MORGAN_WIDGET_URL.
</acceptance_criteria>

<subtasks>
- Initialize Next.js 15 project at apps/website with TypeScript and TailwindCSS 4: Scaffold the Next.js 15 App Router project with all required dependencies installed and baseline configuration files in place.
- Create shared design tokens Tailwind config package at packages/design-tokens: Build the shared Tailwind configuration package defining the Sigma-1 brand palette, typography, spacing, and border radius tokens, importable by apps/website.
- Initialize shadcn/ui with New York style and install all required components: Run shadcn/ui initialization configured for New York style and TailwindCSS 4, then install all component primitives needed across the site.
- Build root layout with sidebar navigation and mobile hamburger menu: Implement app/layout.tsx with the shadcn/ui Sidebar for desktop and Sheet-based hamburger menu for mobile, plus nav links to all pages.
- Implement Effect ApiClient service and Effect.Schema type definitions for all backend response shapes: Create lib/api.ts with an Effect.Service-based ApiClient and define Effect.Schema schemas for all backend data types used across the site.
- Build hero page (app/page.tsx) with value proposition and CTAs: Implement the homepage with a hero section, Sigma-1 value proposition copy, and CTA buttons linking to /equipment and /quote.
- Build equipment catalog page (app/equipment/page.tsx) with category filter and search: Implement the equipment catalog browser as a server-rendered page with client-side category filtering, debounced search, and product cards displaying image, name, day rate, and availability badge.
- Build product detail page (app/equipment/[id]/page.tsx) with real-time availability date picker: Implement the individual product detail page with server-side product data fetch and client-side date range picker that updates the availability badge in real time via TanStack Query.
- Build quote builder page (app/quote/page.tsx) with Effect.Schema validation and RMS submission: Implement the self-service quote builder as a client component with pre-populated items from localStorage, customer info form with Effect.Schema validation, and RMS opportunity creation on submit.
- Build portfolio page (app/portfolio/page.tsx) with masonry grid and lightbox: Implement the event portfolio page fetching published social posts and displaying them in a masonry grid with a lightbox modal for full-size image viewing.
- Implement AI-native routes (llms.txt, llms-full) and Schema.org JSON-LD in layout: Create the /llms.txt and /llms-full route handlers for AI crawler discoverability, and inject Schema.org Organization/LocalBusiness JSON-LD into the root layout.
- Embed Morgan web chat widget in root layout via next/script: Add the Morgan chat widget JavaScript to the root layout using next/script with afterInteractive strategy, sourcing the URL from the NEXT_PUBLIC_MORGAN_WIDGET_URL environment variable.
- Configure Cloudflare Pages deployment with wrangler.toml and GitHub Actions workflow: Set up the Cloudflare Pages deployment pipeline using @cloudflare/next-on-pages, wrangler.toml configuration, and a GitHub Actions workflow that builds and deploys on push to main.
</subtasks>