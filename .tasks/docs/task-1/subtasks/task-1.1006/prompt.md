Implement subtask 1006: Provision Kubernetes Secrets for external API keys

## Objective
Create Kubernetes Secrets for all third-party API keys required by Sigma-1 services: Stripe, OpenCorporates, LinkedIn, Google APIs, and any other external integrations. Use a consistent naming convention and namespace placement.

## Steps
1. Define a naming convention for secrets: `sigma1-<provider>-credentials` (e.g., `sigma1-stripe-credentials`, `sigma1-opencorporates-credentials`, `sigma1-linkedin-credentials`, `sigma1-google-credentials`).
2. Create each secret in the `sigma1` namespace as Opaque type.
3. For Stripe: keys `STRIPE_SECRET_KEY`, `STRIPE_PUBLISHABLE_KEY`, `STRIPE_WEBHOOK_SECRET`.
4. For OpenCorporates: key `OPENCORPORATES_API_KEY`.
5. For LinkedIn: keys `LINKEDIN_CLIENT_ID`, `LINKEDIN_CLIENT_SECRET`.
6. For Google: keys `GOOGLE_CLIENT_ID`, `GOOGLE_CLIENT_SECRET`, `GOOGLE_API_KEY`.
7. Create placeholder values (clearly marked as `REPLACE_ME_<key>`) so that the manifests can be applied without real keys during dev bootstrap. Document which keys need to be replaced before production.
8. If using External Secrets Operator (dp-14), instead create ExternalSecret CRs pointing to a secrets backend (e.g., AWS Secrets Manager, Vault). Otherwise, create static Kubernetes Secrets.
9. Apply all secret manifests.

## Validation
Verify each secret exists in the sigma1 namespace using `kubectl get secrets -n sigma1`. For each secret, confirm the expected keys are present (without inspecting values) using `kubectl get secret <name> -n sigma1 -o jsonpath='{.data}' | jq 'keys'`. Verify secrets are accessible by the default service account in the sigma1 namespace.