Implement subtask 9004: Configure Cloudflare CDN for static assets and SSL termination

## Objective
Set up Cloudflare CDN to cache and serve static assets from the web frontend, configure SSL/TLS termination at Cloudflare edge, and set appropriate cache rules.

## Steps
1. Configure the Cloudflare zone DNS records to point the domain to the cluster's external IP or Cloudflare Tunnel. 2. Enable Cloudflare SSL mode to 'Full (Strict)' to ensure end-to-end encryption. 3. Create Page Rules or Cache Rules for static asset paths (e.g., /static/*, /_next/static/*) with aggressive caching (Edge TTL: 1 month, Browser TTL: 1 week). 4. Enable Auto Minify for JS/CSS/HTML. 5. Configure Cloudflare WAF basic rules for common attack vectors. 6. Set up an origin certificate from Cloudflare and install it as a Kubernetes TLS Secret for the origin server. 7. Verify CDN cache hit rate via Cloudflare analytics dashboard.

## Validation
Request a static asset and verify the `cf-cache-status: HIT` response header on subsequent requests. Confirm SSL certificate is valid via browser inspection (no mixed content warnings). Verify Cloudflare analytics show cache hit ratio > 80% for static assets after warm-up. Test that non-cached API routes pass through to origin correctly.