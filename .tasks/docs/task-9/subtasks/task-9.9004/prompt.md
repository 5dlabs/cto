Implement subtask 9004: Configure Cloudflare CDN and TLS for all public endpoints

## Objective
Set up Cloudflare DNS, CDN caching rules, and TLS certificates for all public-facing domains and subdomains.

## Steps
1. Add all public domain DNS records in Cloudflare (proxied orange-cloud for CDN). 2. Set SSL/TLS mode to 'Full (Strict)' in Cloudflare dashboard. 3. Create Cloudflare Origin CA certificates and store them as Kubernetes TLS secrets for origin server verification. 4. Configure Cloudflare Page Rules or Cache Rules for static assets (e.g., `/static/*`, `/assets/*`) with aggressive caching. 5. Set 'Always Use HTTPS' and enable HSTS with `max-age=31536000; includeSubDomains`. 6. Configure minimum TLS version to 1.2. 7. Test that all public endpoints resolve through Cloudflare with valid TLS certificates.

## Validation
Run `curl -vI https://<domain>` and verify the certificate chain includes Cloudflare. Confirm HSTS header is present. Verify HTTP requests redirect to HTTPS. Check that static assets return `cf-cache-status: HIT` after second request.