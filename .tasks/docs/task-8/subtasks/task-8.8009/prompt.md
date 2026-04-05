Implement subtask 8009: Configure Cloudflare Pages deployment and end-to-end testing

## Objective
Set up Cloudflare Pages deployment for the Next.js 15 application, configure environment variables, custom domain, and run end-to-end tests against the deployed site.

## Steps
1. Configure `wrangler.toml` or Cloudflare Pages dashboard for the Next.js project.
2. Set up the build command (`next build`) and output directory.
3. Configure environment variables in Cloudflare Pages: API base URLs for Equipment Catalog, Social Media Engine, Morgan WebSocket endpoint.
4. Set up custom domain and DNS records.
5. Deploy and verify the site is accessible at the custom domain.
6. Run end-to-end checks:
   - Home page loads and all links work.
   - /equipment fetches and displays data from the live API.
   - /equipment/:id shows correct details.
   - /quote form submits successfully.
   - Chat widget connects to Morgan.
   - /portfolio displays content.
   - /llms.txt and /llms-full are accessible.
7. Verify HTTPS is active and headers are correct (HSTS, CSP).
8. Test performance with Lighthouse (target: 90+ performance score).

## Validation
Site is live at custom domain over HTTPS. All pages load without errors. API integrations return live data. Quote submission works. Chat widget connects. Lighthouse performance score is 90+. /llms.txt is accessible. No console errors in production.