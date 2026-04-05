Implement subtask 9003: Configure Cloudflare CDN for static assets and SSL termination

## Objective
Set up Cloudflare CDN to cache and serve static assets globally with SSL/TLS termination at the edge. Configure cache rules, page rules, and SSL mode for all public-facing domains.

## Steps
1. Ensure DNS for all public-facing domains is proxied through Cloudflare (orange cloud).
2. Set SSL/TLS mode to 'Full (Strict)' in Cloudflare dashboard or via API/Terraform.
3. Configure cache rules for static asset paths (e.g., `/static/*`, `/_next/static/*`, image/font/CSS/JS extensions) with long TTLs.
4. Set appropriate cache-control headers in application responses for static assets.
5. Configure page rules or cache rules to bypass cache for API endpoints and dynamic routes.
6. Enable HSTS via Cloudflare settings.
7. Enable Brotli compression.
8. Document the Cloudflare configuration including zone ID, cache rules, and SSL settings in the infra repo.

## Validation
Verify SSL/TLS is enforced by hitting public domains over HTTPS and confirming valid certificates. Confirm HTTP requests are redirected to HTTPS. Verify static assets return `cf-cache-status: HIT` header after first request. Verify API endpoints return `cf-cache-status: DYNAMIC` or `BYPASS`. Run SSL Labs test and confirm A+ rating.