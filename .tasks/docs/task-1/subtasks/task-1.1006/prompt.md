Implement subtask 1006: Create external service Kubernetes Secrets

## Objective
Create Kubernetes Secrets for all external third-party API credentials: ElevenLabs, Twilio, Stripe, OpenCorporates, LinkedIn, and Google Reviews.

## Steps
1. Create a Secret named sigma1-elevenlabs-secret in sigma1 namespace with keys: ELEVENLABS_API_KEY, ELEVENLABS_VOICE_ID.
2. Create a Secret named sigma1-twilio-secret with keys: TWILIO_ACCOUNT_SID, TWILIO_AUTH_TOKEN, TWILIO_PHONE_NUMBER.
3. Create a Secret named sigma1-stripe-secret with keys: STRIPE_SECRET_KEY, STRIPE_WEBHOOK_SECRET, STRIPE_PUBLISHABLE_KEY.
4. Create a Secret named sigma1-opencorporates-secret with key: OPENCORPORATES_API_TOKEN.
5. Create a Secret named sigma1-linkedin-secret with keys: LINKEDIN_CLIENT_ID, LINKEDIN_CLIENT_SECRET.
6. Create a Secret named sigma1-google-reviews-secret with keys: GOOGLE_PLACES_API_KEY.
7. Use placeholder values for now; document which keys must be populated with real credentials before production.
8. Apply all Secret manifests.

## Validation
Verify each Secret exists in the sigma1 namespace with all expected keys using `kubectl get secret <name> -o jsonpath`. Confirm no secrets are missing keys. Verify secrets are accessible from a test pod with the appropriate ServiceAccount.