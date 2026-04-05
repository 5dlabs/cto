Implement subtask 8008: Accessibility audit, responsive design polish, and Cloudflare Pages deployment

## Objective
Conduct an accessibility audit (WCAG 2.1 AA) across all pages, polish responsive design for all breakpoints, and configure deployment to Cloudflare Pages with environment variables and build settings.

## Steps
Step 1: Run automated accessibility audit tools (axe-core, Lighthouse) across all pages; document violations. Step 2: Fix accessibility issues: missing alt text, insufficient color contrast, keyboard navigation gaps, missing ARIA labels, focus management in modals (chat widget, lightbox). Step 3: Test with screen readers (VoiceOver/NVDA) on key flows: home → catalog → detail → quote builder → submit. Step 4: Verify responsive design across breakpoints: 320px (small mobile), 375px (iPhone), 768px (tablet), 1024px (laptop), 1440px (desktop). Fix layout issues, overflow, touch targets. Step 5: Configure Cloudflare Pages project: connect Git repo, set build command (`next build`), output directory, Node.js version, and environment variables (API URLs, etc.). Step 6: Configure the `@cloudflare/next-on-pages` adapter for Next.js 15 compatibility. Step 7: Deploy to Cloudflare Pages and verify all pages are reachable, API calls work through the production domain, and assets load from Cloudflare CDN.

## Validation
Lighthouse accessibility score ≥90 on all pages; no critical WCAG 2.1 AA violations; responsive layout is correct across all tested breakpoints; Cloudflare Pages deployment succeeds; production site loads all pages, API calls return data, chat widget connects, and images load via CDN.