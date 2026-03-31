Implement subtask 9003: Configure Cloudflare CDN for static assets and SSL/TLS termination

## Objective
Set up Cloudflare CDN to cache and serve static assets with SSL/TLS termination at the edge. Configure caching rules, SSL mode, and security headers.

## Steps
1. Configure Cloudflare DNS zone with proxied (orange cloud) A/CNAME records for the web domain.
2. Set SSL/TLS mode to 'Full (Strict)' to enforce encrypted connections between Cloudflare and origin.
3. Create Page Rules or Cache Rules for static asset paths (e.g., `/_next/static/*`, `/assets/*`) with cache TTL of 1 month and `Cache-Control: public, max-age=2592000, immutable`.
4. Enable 'Always Use HTTPS' and 'Automatic HTTPS Rewrites'.
5. Configure minimum TLS version to 1.2.
6. Add security headers via Cloudflare Transform Rules: `Strict-Transport-Security`, `X-Content-Type-Options: nosniff`, `X-Frame-Options: DENY`.
7. Enable Brotli compression.
8. Document the Cloudflare configuration in the repo (as IaC if using Cloudflare Terraform provider, or as runbook if manual).

## Validation
Verify `curl -I https://<domain>/_next/static/test.js` returns `cf-cache-status: HIT` on second request. Confirm SSL Labs test shows A+ rating. Verify all HTTP requests redirect to HTTPS. Confirm security headers are present in response. Measure static asset latency from multiple geolocations is <100ms.