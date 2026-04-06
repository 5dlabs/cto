Implement subtask 1006: Create external API credential secrets

## Objective
Create Kubernetes secrets for all external API credentials: Stripe, OpenCorporates, LinkedIn, Google Reviews, Instagram, and Facebook.

## Steps
1. Create individual Kubernetes Secrets in the databases namespace (centralized, referenced by downstream services):
   - sigma1-stripe-credentials: STRIPE_API_KEY, STRIPE_WEBHOOK_SECRET
   - sigma1-opencorporates-credentials: OPENCORPORATES_API_KEY
   - sigma1-linkedin-credentials: LINKEDIN_CLIENT_ID, LINKEDIN_CLIENT_SECRET, LINKEDIN_ACCESS_TOKEN
   - sigma1-google-reviews-credentials: GOOGLE_REVIEWS_API_KEY, GOOGLE_PLACES_ID
   - sigma1-instagram-credentials: INSTAGRAM_ACCESS_TOKEN, INSTAGRAM_BUSINESS_ID
   - sigma1-facebook-credentials: FACEBOOK_PAGE_ACCESS_TOKEN, FACEBOOK_PAGE_ID
2. Use placeholder values for now (to be replaced with real credentials before production).
3. Label all secrets with app.kubernetes.io/part-of=sigma1-infra for easy discovery.
4. Document each secret's expected keys in a README or annotation.

## Validation
Run `kubectl get secrets -n databases -l app.kubernetes.io/part-of=sigma1-infra` and confirm all 6 secrets are listed. For each secret, verify expected keys are present via `kubectl get secret <name> -n databases -o jsonpath='{.data}' | jq 'keys'`.