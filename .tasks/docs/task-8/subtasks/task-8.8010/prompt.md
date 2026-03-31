Implement subtask 8010: Configure Cloudflare Pages deployment and Lighthouse optimization

## Objective
Set up Cloudflare Pages deployment configuration, optimize the application for Lighthouse performance score >90, and ensure production build is correct.

## Steps
1. Install and configure `@cloudflare/next-on-pages` for Cloudflare Pages compatibility with Next.js 15 App Router.
2. Create `wrangler.toml` with project name, compatibility date, and any environment variable bindings.
3. Configure `next.config.ts` for Cloudflare Pages: set `output` mode if needed, configure image optimization (use Cloudflare Images or `unoptimized` if not available).
4. Optimize Lighthouse Performance: implement `next/image` for all images with proper width/height/priority attributes, add font-display: swap for web fonts, ensure critical CSS is inlined (TailwindCSS 4 handles this), lazy-load below-fold components with `dynamic()` or `React.lazy()`.
5. Optimize Lighthouse Accessibility: ensure all images have alt text, form labels are associated, color contrast meets WCAG AA, focus management works for keyboard navigation.
6. Optimize Lighthouse SEO: verify all pages have unique titles and meta descriptions, canonical URLs, proper heading hierarchy.
7. Optimize Lighthouse Best Practices: ensure HTTPS, no mixed content, no console errors.
8. Add `_headers` file for security headers (CSP, X-Frame-Options, etc.) on Cloudflare Pages.
9. Run `npm run build` and verify no build errors. Test with `wrangler pages dev` locally.
10. Document deployment steps in README.

## Validation
Production build completes without errors. `wrangler pages dev` serves the site locally. Lighthouse audit on homepage, /equipment, and /quote all score >90 in Performance, Accessibility, SEO, and Best Practices. All pages load within 3 seconds on simulated 3G. No console errors in production build.