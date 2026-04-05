Implement subtask 9003: Configure Cloudflare CDN for static assets and SSL termination

## Objective
Set up Cloudflare CDN to serve static assets (product images, frontend bundles) with proper cache rules and configure SSL/TLS termination at the Cloudflare edge.

## Steps
1. Configure Cloudflare DNS records for the production domain pointing to the cluster ingress (or Cloudflare Tunnel).
2. Enable Cloudflare SSL/TLS in Full (Strict) mode with an origin certificate installed on the cluster.
3. Create Cloudflare page rules or cache rules for static asset paths (e.g., `/static/*`, `/images/*`, `/_next/static/*`) with appropriate TTLs.
4. If using Cloudflare R2 for object storage, configure a custom domain on the R2 bucket and set cache headers.
5. Enable Brotli/gzip compression at the edge.
6. Configure security headers (HSTS, X-Frame-Options, CSP) via Cloudflare transform rules.
7. Document the CDN configuration including cache purge procedures.

## Validation
Verify static assets return `cf-cache-status: HIT` header after second request; confirm TLS certificate is valid and HSTS header is present; verify Brotli compression is active; confirm origin is not directly accessible bypassing CDN.