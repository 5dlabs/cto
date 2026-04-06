Implement subtask 8009: Configure and deploy to Cloudflare Pages

## Objective
Set up the Cloudflare Pages project, configure build settings for Next.js 15, set environment variables for backend API endpoints, and deploy the production build.

## Steps
1. Create a Cloudflare Pages project linked to the repository.
2. Configure the build command (`next build`) and output directory.
3. Set up the `@cloudflare/next-on-pages` adapter if needed for edge runtime compatibility.
4. Configure environment variables in Cloudflare Pages dashboard: all backend API URLs (matching sigma1-infra-endpoints), Morgan WebSocket URL, and any public keys.
5. Configure custom domain and DNS if applicable.
6. Run a production build locally first to catch any build errors.
7. Deploy via Cloudflare Pages (git push trigger or wrangler CLI).
8. Verify all pages load correctly on the deployed URL.
9. Test that API integrations work from the deployed environment (CORS, network connectivity).
10. Verify Lighthouse performance score >90 on the deployed site.

## Validation
Cloudflare Pages build succeeds; deployed site is accessible at the configured URL; all pages render correctly; API calls to backend services succeed (no CORS or network errors); Morgan chat widget connects; Lighthouse performance score >90; Lighthouse accessibility score >90.