Implement subtask 8013: Configure Cloudflare Pages deployment with environment variables and edge caching

## Objective
Set up the Cloudflare Pages deployment configuration including wrangler.toml or Cloudflare Pages project settings, environment variable configuration for API base URLs, and edge caching settings for static pages.

## Steps
1. Create `wrangler.toml` at project root (if using Cloudflare Pages with wrangler):
   - Set `name`, `compatibility_date`, `pages_build_output_dir = '.vercel/output/static'` or configure for Next.js on Cloudflare Pages adapter.
   - Or: configure via Cloudflare dashboard and document settings in a `DEPLOYMENT.md`.
2. Install `@cloudflare/next-on-pages` adapter if needed for Next.js on Cloudflare Pages.
3. Configure environment variables in Cloudflare Pages dashboard (document in .env.local.example):
   - `NEXT_PUBLIC_API_BASE_URL`: Equipment/Quote API base URL.
   - `NEXT_PUBLIC_WS_URL`: Morgan WebSocket URL.
   - Any other API keys needed.
4. Configure `next.config.ts`:
   - Output mode compatible with Cloudflare Pages.
   - Image optimization: configure `remotePatterns` for R2 CDN image URLs.
   - Headers: add cache-control headers for static assets (long TTL), dynamic pages (short TTL).
5. Add build script to `package.json`: `"deploy": "npx @cloudflare/next-on-pages"` or equivalent.
6. Create a GitHub Actions workflow or document manual deployment steps.

## Validation
Run the build command (`next build` + Cloudflare adapter) and verify it completes without errors. Verify the output directory contains expected static assets. Verify environment variables are properly referenced (not hardcoded). Deploy to a preview environment on Cloudflare Pages and confirm the site loads with correct API connections.