Implement subtask 8013: Performance optimization and Lighthouse CI targets

## Objective
Optimize all pages for Lighthouse Performance > 90 and Accessibility > 95. Configure Next.js Image with R2 CDN loader, static generation for appropriate pages, code splitting, and bundle analysis.

## Steps
1. Image optimization:
   - Create custom Next.js Image loader for R2 CDN (`lib/image-loader.ts`): construct URLs with width/quality params if R2/Cloudflare supports image transformations, otherwise serve original with size hints.
   - Ensure all images use `<Image>` component with appropriate `sizes` prop for responsive loading.
   - Priority loading for above-the-fold images (hero, first few product cards).
2. Static generation:
   - Home page: fully static (`force-static` or default).
   - Portfolio page: static or ISR.
   - Equipment listing: dynamic (search params driven).
   - Product detail: consider `generateStaticParams` for top products, fallback dynamic for rest.
3. Code splitting:
   - ChatWidget loaded via `next/dynamic` with `ssr: false` (no SSR needed for chat).
   - AvailabilityCalendar loaded dynamically on product detail page.
   - TanStack Query DevTools only in dev.
4. Bundle analysis:
   - Run `@next/bundle-analyzer` to identify large chunks.
   - Ensure Effect library is tree-shaken (import from specific submodules).
5. Font optimization: use `next/font` for Google Fonts or local fonts, preload.
6. Prefetching: Next.js Link prefetch for likely navigation targets.
7. Run Lighthouse on: / (home), /equipment, /equipment/:id. Target: Performance > 90, Accessibility > 95, Best Practices > 90.
8. Set up Lighthouse CI config (`.lighthouserc.js`) for CI pipeline integration.

## Validation
Run Lighthouse CI on home page: Performance > 90, Accessibility > 95, Best Practices > 90. Run on /equipment page: same targets. Verify bundle analyzer output shows no unexpectedly large chunks (> 200KB gzipped). Verify ChatWidget does not appear in SSR output (dynamic import with ssr:false). Verify images on equipment page use R2 CDN URLs with proper dimensions.