Implement subtask 1006: Create ExternalSecret CRs for third-party API keys

## Objective
Create ExternalSecret custom resources for all third-party service API keys as placeholders, referencing the chosen external secrets backend.

## Steps
1. Ensure a `ClusterSecretStore` or `SecretStore` CR exists in the sigma1 namespace pointing to the chosen secrets backend (see decision point).
2. Create the following ExternalSecret CRs in namespace `sigma1`:
   a. `sigma1-stripe-keys.yaml`:
      - Secret keys: `STRIPE_PUBLISHABLE_KEY`, `STRIPE_SECRET_KEY`
      - `refreshInterval: 1h`
   b. `sigma1-opencorporates-key.yaml`:
      - Secret key: `OPENCORPORATES_API_KEY`
   c. `sigma1-social-api-keys.yaml`:
      - Secret keys: `INSTAGRAM_ACCESS_TOKEN`, `LINKEDIN_ACCESS_TOKEN`, `FACEBOOK_ACCESS_TOKEN`
   d. `sigma1-elevenlabs-key.yaml`:
      - Secret key: `ELEVENLABS_API_KEY`
   e. `sigma1-twilio-keys.yaml`:
      - Secret keys: `TWILIO_ACCOUNT_SID`, `TWILIO_AUTH_TOKEN`, `TWILIO_PHONE_NUMBER`
   f. `sigma1-openai-key.yaml`:
      - Secret key: `OPENAI_API_KEY`
   g. `sigma1-google-calendar-creds.yaml`:
      - Secret keys: `GOOGLE_CALENDAR_CLIENT_ID`, `GOOGLE_CALENDAR_CLIENT_SECRET`, `GOOGLE_CALENDAR_REFRESH_TOKEN`
3. For dev/bootstrap, seed placeholder values in the external secrets backend so the ExternalSecret CRs can sync successfully.
4. Apply all ExternalSecret manifests.

## Validation
All 7 ExternalSecret CRs exist in sigma1 namespace: `kubectl get externalsecrets -n sigma1`. Each reports `SecretSynced` condition as True. Corresponding Kubernetes Secrets are created with the expected keys (values can be placeholders). `kubectl get secret sigma1-stripe-keys -n sigma1 -o jsonpath='{.data}'` shows base64-encoded values for both keys.