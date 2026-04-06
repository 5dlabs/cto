Implement subtask 9002: Configure Cloudflare CDN and TLS for all public-facing endpoints

## Objective
Set up Cloudflare DNS records, enable CDN caching for static assets, and configure TLS (Full Strict mode) for all public-facing domains and subdomains.

## Steps
1) Create or update Cloudflare DNS records (A/CNAME) for each public endpoint (frontend app domain, API domain, any webhook endpoints). 2) Enable Cloudflare proxy (orange cloud) on each record to activate CDN. 3) Set SSL/TLS mode to 'Full (Strict)' in Cloudflare dashboard or via API. 4) Generate a Cloudflare Origin CA certificate and install it as a Kubernetes TLS Secret for backend services to present to Cloudflare. 5) Configure Cloudflare Page Rules or Cache Rules: cache static assets (JS, CSS, images) with long TTLs, bypass cache for API routes. 6) Enable 'Always Use HTTPS' and HSTS headers in Cloudflare. 7) Store Cloudflare API tokens as Kubernetes Secrets for any automation. 8) Document the DNS records and Cloudflare settings in the project's infrastructure README.

## Validation
Verify all public domains resolve correctly via `dig` or `nslookup`. Confirm HTTPS works end-to-end by curling each endpoint and checking for valid TLS certificate (Cloudflare-issued edge cert + origin cert chain). Verify CDN is active by checking `cf-cache-status` response header on static asset requests (should be HIT after first request). Confirm HTTP requests redirect to HTTPS. Run SSL Labs test against public domains and verify A/A+ rating.