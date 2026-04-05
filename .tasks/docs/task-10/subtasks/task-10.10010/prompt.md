Implement subtask 10010: Ingress and CDN: configure CDN caching rules for equipment images vs API responses

## Objective
Set up Cloudflare CDN caching rules: 1-year cache for equipment images, no-cache or short TTL for API responses.

## Steps
Step-by-step:
1. In the Cloudflare dashboard (or via Terraform/API if IaC is used), create Page Rules or Cache Rules:
   - Rule 1: `*sigma1.example.com/images/*` → Cache Level: Cache Everything, Edge Cache TTL: 1 year (31536000s), Browser Cache TTL: 1 year.
   - Rule 2: `*sigma1.example.com/api/*` → Cache Level: Bypass (or Standard with `Cache-Control: no-store` respected).
2. Ensure equipment-catalog service sets appropriate `Cache-Control` headers on image responses: `public, max-age=31536000, immutable`.
3. Ensure all API endpoints set `Cache-Control: no-store` or `private, no-cache`.
4. Document the mTLS Phase 2 decision: create `docs/mtls-phase2.md` explaining that internal service-to-service mTLS via cert-manager is deferred, with a brief architecture sketch for future implementation.

## Validation
Fetch an equipment image URL via curl with `-I` flag, verify `CF-Cache-Status: HIT` on second request and appropriate cache headers. Fetch an API endpoint, verify `CF-Cache-Status: DYNAMIC` or `BYPASS` and no caching.