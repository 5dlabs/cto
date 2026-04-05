Implement subtask 1007: Create Kubernetes Secrets for R2, Stripe, external APIs, and inter-service keys

## Objective
Create the remaining four Kubernetes Secrets: sigma1-r2-credentials, sigma1-stripe-credentials, sigma1-external-apis, and sigma1-service-api-keys in the sigma1 namespace.

## Steps
1. Create Secret `sigma1-r2-credentials` with keys: `R2_ACCESS_KEY_ID`, `R2_SECRET_ACCESS_KEY`, `R2_BUCKET`, `R2_ENDPOINT`.
2. Create Secret `sigma1-stripe-credentials` with placeholder keys: `STRIPE_SECRET_KEY`, `STRIPE_PUBLISHABLE_KEY`, `STRIPE_WEBHOOK_SECRET`.
3. Create Secret `sigma1-external-apis` with keys: `OPENCORPORATES_API_KEY`, `ELEVENLABS_API_KEY`, `TWILIO_ACCOUNT_SID`, `TWILIO_AUTH_TOKEN`, `TWILIO_PHONE_NUMBER`, `GOOGLE_CALENDAR_CREDENTIALS` (JSON service account key).
4. Create Secret `sigma1-service-api-keys` with pre-shared API keys for inter-service auth: one key per service pair that needs to communicate (e.g., `CATALOG_TO_RMS_KEY`, `RMS_TO_FINANCE_KEY`, etc. per D7 resolution).
5. For placeholder values, use clearly marked strings like `REPLACE_ME_<service>` so they're easy to find.
6. Apply all four Secret YAMLs.

## Validation
`kubectl get secrets -n sigma1` lists all four secrets. Each secret has non-empty data keys: `kubectl get secret <name> -n sigma1 -o json | jq '.data | length'` returns expected count. No key has an empty value.