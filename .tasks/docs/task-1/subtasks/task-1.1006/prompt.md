Implement subtask 1006: Create external service secrets for third-party APIs

## Objective
Create Kubernetes Secrets in the sigma1 namespace for all external API credentials: ElevenLabs, Twilio, Stripe, OpenCorporates, LinkedIn, Google Reviews, and credit bureau APIs.

## Steps
1. Define a Secret manifest for each external service with appropriate keys:
   - elevenlabs-api: ELEVENLABS_API_KEY, ELEVENLABS_VOICE_ID
   - twilio-api: TWILIO_ACCOUNT_SID, TWILIO_AUTH_TOKEN, TWILIO_PHONE_NUMBER
   - stripe-api: STRIPE_SECRET_KEY, STRIPE_WEBHOOK_SECRET, STRIPE_PUBLISHABLE_KEY
   - opencorporates-api: OPENCORPORATES_API_KEY
   - linkedin-api: LINKEDIN_CLIENT_ID, LINKEDIN_CLIENT_SECRET
   - google-reviews-api: GOOGLE_API_KEY, GOOGLE_PLACE_ID
   - credit-api: CREDIT_API_KEY, CREDIT_API_ENDPOINT
2. Use placeholder values for now (to be replaced with real credentials before production).
3. Apply all Secrets to the sigma1 namespace.
4. Verify each Secret exists and contains the expected keys.
5. Record each secret name and its keys for the ConfigMap references.

## Validation
Run `kubectl get secrets -n sigma1` and confirm all 7 external service secrets exist; for each secret, verify the expected keys are present using `kubectl get secret <name> -n sigma1 -o jsonpath='{.data}' | jq keys`.