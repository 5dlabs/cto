Implement subtask 8014: Configure Cloudflare Pages deployment with @cloudflare/next-on-pages

## Objective
Set up the project for Cloudflare Pages deployment using the @cloudflare/next-on-pages adapter. Configure edge runtime compatibility, environment variables, build output, and wrangler configuration.

## Steps
1. Install `@cloudflare/next-on-pages` and `wrangler` as dev dependencies.
2. Add build script: `"build:cf": "npx @cloudflare/next-on-pages"` to package.json.
3. Create `wrangler.toml` (or configure via Cloudflare dashboard):
   - `compatibility_date` set to current.
   - `compatibility_flags = ["nodejs_compat"]` if needed.
   - Environment variables: `NEXT_PUBLIC_API_BASE_URL`, `NEXT_PUBLIC_CDN_URL`, `NEXT_PUBLIC_WS_URL`.
4. Edge runtime compatibility:
   - Audit all route handlers and middleware for Node.js-only APIs.
   - Add `export const runtime = 'edge'` to routes that can run on edge.
   - For routes requiring Node.js (if any), ensure they work in Cloudflare Workers node compat mode.
5. Verify `next.config.ts` is compatible: no `output: 'standalone'`, no incompatible plugins.
6. Test build: run `npx @cloudflare/next-on-pages` locally, verify it completes without errors.
7. Test local preview: `npx wrangler pages dev .vercel/output/static` to verify pages serve correctly.
8. Document deployment steps in README: environment variable setup, build command, deployment command.
9. Create `_headers` file in public/ for any custom Cloudflare headers (caching, security headers).

## Validation
Run `npx @cloudflare/next-on-pages` and verify build completes with exit code 0. Run local preview with wrangler and verify home page loads, equipment page loads with mock API, and static assets (images, fonts) serve correctly. Verify environment variables are accessible in both server and client components.