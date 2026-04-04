Implement subtask 9010: Configure CDN caching for static dashboard assets

## Objective
If the Next.js dashboard (Task 7) is deployed and exposed via Ingress, configure CDN caching for static assets while ensuring API routes bypass the cache.

## Steps
Step-by-step:
1. Determine if Task 7 (dashboard) has been deployed and is exposed via the Ingress. If not, this subtask is deferred.
2. Based on the chosen CDN provider, configure caching rules:
   - Cache: `/_next/static/*`, `/static/*`, `/favicon.ico` — set Cache-Control max-age=31536000 (immutable fingerprinted assets)
   - No cache: `/api/*`, `/health`, `/ready` — set Cache-Control: no-store
3. If using Cloudflare: configure Page Rules or Cache Rules for the hostname.
   If using nginx ingress annotations: add `nginx.ingress.kubernetes.io/server-snippet` with `location` blocks that set appropriate cache headers.
4. Alternatively, configure cache-control headers in the Next.js application itself (preferred — the CDN respects origin headers).
5. Validate: request a static asset and check response headers include appropriate Cache-Control. Request an API route and confirm no caching headers.

## Validation
`curl -I https://<host>/_next/static/<hash>.js` returns `Cache-Control: public, max-age=31536000, immutable`. `curl -I https://<host>/api/health` returns `Cache-Control: no-store` or no cache header. CDN cache HIT headers appear on subsequent requests for static assets.