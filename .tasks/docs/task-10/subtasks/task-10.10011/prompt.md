Implement subtask 10011: Configure Cloudflare WAF rate limiting and bot protection rules

## Objective
Define Cloudflare WAF rules for rate limiting public API endpoints (100 req/min per IP) and bot protection on the website, expressed as Terraform resources or Cloudflare API configurations.

## Steps
1. Define Cloudflare rate limiting rules (via Terraform `cloudflare_rate_limit` resource or Cloudflare dashboard config-as-code):
   - Rule 1: Rate limit on `api.sigma-1.com/*` — 100 requests per minute per IP, response 429 with JSON body `{ error: 'rate_limited', retry_after: 60 }`.
   - Rule 2: Stricter rate limit on authentication endpoints (if any) — 20 requests per minute per IP.
2. Configure Cloudflare Bot Management or Bot Fight Mode on the website domain:
   - Enable bot score threshold to challenge suspicious traffic.
   - Allow known good bots (Googlebot, etc.).
3. If using Terraform:
   - Create `cloudflare-waf.tf` with the rate limit resources.
   - Store Cloudflare API token as ExternalSecret.
4. If manual/dashboard:
   - Document the exact rule configurations in the ops runbook ConfigMap.
5. Ensure rate limit responses include `Retry-After` header.

## Validation
Send 101 requests from a single IP to `api.sigma-1.com` within 1 minute — verify request 101 receives a 429 response. Verify the 429 response body contains the expected JSON. Verify the `Retry-After` header is present. Verify bot protection is active by checking Cloudflare dashboard status or Terraform state.