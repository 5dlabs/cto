Implement subtask 9004: Configure Cloudflare CDN for static assets and SSL termination

## Objective
Set up Cloudflare CDN to cache and serve static assets (images, JS, CSS) with SSL termination, appropriate cache rules, and security headers for the web frontend.

## Steps
1. Ensure the domain is proxied through Cloudflare (orange cloud enabled).
2. Configure SSL/TLS mode to 'Full (Strict)' in Cloudflare dashboard or via Terraform/API.
3. Create Page Rules or Cache Rules for static asset paths (e.g., `/static/*`, `/_next/static/*`) with aggressive caching (Cache-Control: public, max-age=31536000, immutable).
4. Enable Auto Minify for JS/CSS if desired.
5. Configure security headers via Cloudflare Transform Rules: HSTS, X-Content-Type-Options, X-Frame-Options, Referrer-Policy.
6. Enable Brotli compression.
7. Verify CDN is serving cached content by checking `cf-cache-status` response header.

## Validation
Curl a static asset URL and verify `cf-cache-status: HIT` header is present on second request. Verify SSL certificate is valid and issued by Cloudflare. Verify HSTS and security headers are present in responses. Run SSL Labs test and confirm A+ rating.