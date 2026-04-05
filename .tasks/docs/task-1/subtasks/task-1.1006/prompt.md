Implement subtask 1006: Provision third-party API secrets across namespaces

## Objective
Create Kubernetes Secrets for all third-party API keys (Stripe, OpenCorporates, LinkedIn, Google, etc.) in their respective namespaces so downstream services can consume them securely.

## Steps
1. Define a manifest per secret with placeholder values (to be filled by operators) under infra/secrets/.
2. Create 'stripe-api-keys' Secret in sigma1 namespace: STRIPE_SECRET_KEY, STRIPE_PUBLISHABLE_KEY, STRIPE_WEBHOOK_SECRET.
3. Create 'opencorporates-api' Secret in sigma1 namespace: OPENCORPORATES_API_KEY.
4. Create 'linkedin-api' Secret in social namespace: LINKEDIN_CLIENT_ID, LINKEDIN_CLIENT_SECRET.
5. Create 'google-api' Secret in sigma1 namespace: GOOGLE_API_KEY, GOOGLE_CLIENT_ID, GOOGLE_CLIENT_SECRET.
6. Annotate each secret with 'sigma1.io/managed-by=infra-bootstrap' for lifecycle tracking.
7. Ensure secrets are of type Opaque and base64-encoded.
8. Add a README documenting each secret name, namespace, expected keys, and which service consumes it.

## Validation
kubectl get secrets -n sigma1 lists stripe-api-keys, opencorporates-api, google-api; kubectl get secrets -n social lists linkedin-api; each secret contains the expected keys (kubectl get secret <name> -o jsonpath='{.data}' shows all required keys).