Implement subtask 9005: Configure Cloudflare Access policies for admin endpoints

## Objective
Set up Cloudflare Access policies to protect admin endpoints and sensitive routes behind authentication.

## Steps
1. Identify admin endpoints that need protection (e.g., Grafana dashboard, ArgoCD UI, any admin API routes).
2. Create Cloudflare Access Application configurations:
   - Define the application domain/path patterns
   - Configure identity provider integration (e.g., GitHub OAuth, Google, or one-time PIN)
   - Set access policies: allow specific email addresses or email domains
3. If using the Cloudflare operator, create Access Application CRs; otherwise, document the Cloudflare dashboard configuration needed.
4. Test that unauthenticated requests to admin endpoints are redirected to the Cloudflare Access login page.
5. Test that authenticated requests pass through successfully.

## Validation
Attempt to access an admin endpoint without authentication — verify redirect to Cloudflare Access login page. Authenticate with an allowed identity — verify access is granted. Attempt authentication with a non-allowed identity — verify access is denied.