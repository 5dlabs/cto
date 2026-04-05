Implement subtask 9006: Configure Cloudflare R2 CDN with custom domain and cache rules

## Objective
Set up the custom domain assets.sigma-1.com for the R2 bucket, configure cache rules for images and thumbnails, and enable Cloudflare Polish for image optimization.

## Steps
1. Configure custom domain for R2 bucket:
   - In Cloudflare dashboard or via API, bind `assets.sigma-1.com` to the sigma1 R2 bucket
   - Verify DNS CNAME record is created
2. Configure cache rules via Cloudflare Page Rules or Cache Rules:
   - Rule 1: `assets.sigma-1.com/images/*` → Cache-Control: `public, max-age=31536000, immutable` (1 year)
   - Rule 2: `assets.sigma-1.com/thumbnails/*` → Cache-Control: `public, max-age=2592000` (30 days)
   - Rule 3: Default for other assets → Cache-Control: `public, max-age=86400` (1 day)
3. Enable Cloudflare Polish:
   - Enable Polish in the Cloudflare zone settings (lossy or lossless based on preference)
   - Enable WebP conversion
4. Document the Cloudflare configuration steps (these may not be fully expressible as Kubernetes CRs).
5. Upload a test image to R2 and verify it is served via `assets.sigma-1.com`.

## Validation
Upload a test image to R2. Request it via `curl -I https://assets.sigma-1.com/images/test.jpg` — verify 200 response with correct Cache-Control header. Request again and verify `CF-Cache-Status: HIT`. Verify `cf-polished` header is present indicating Polish is active.