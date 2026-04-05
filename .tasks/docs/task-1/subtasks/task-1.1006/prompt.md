Implement subtask 1006: Create external service secrets for all third-party integrations

## Objective
Create Kubernetes Secrets containing API keys and credentials for ElevenLabs, Twilio, OpenCorporates, LinkedIn, Google Reviews, and Stripe.

## Steps
1. Create a Secret 'sigma1-elevenlabs' in the 'sigma1' namespace with keys: ELEVENLABS_API_KEY, ELEVENLABS_VOICE_ID. 2. Create a Secret 'sigma1-twilio' with keys: TWILIO_ACCOUNT_SID, TWILIO_AUTH_TOKEN, TWILIO_PHONE_NUMBER. 3. Create a Secret 'sigma1-opencorporates' with keys: OPENCORPORATES_API_KEY. 4. Create a Secret 'sigma1-linkedin' with keys: LINKEDIN_CLIENT_ID, LINKEDIN_CLIENT_SECRET, LINKEDIN_ACCESS_TOKEN. 5. Create a Secret 'sigma1-google-reviews' with keys: GOOGLE_REVIEWS_API_KEY, GOOGLE_PLACE_ID. 6. Create a Secret 'sigma1-stripe' with keys: STRIPE_SECRET_KEY, STRIPE_PUBLISHABLE_KEY, STRIPE_WEBHOOK_SECRET. 7. Use placeholder values initially, with clear documentation for which real values need to be injected. 8. Optionally create a sealed-secrets or external-secrets template for production rotation.

## Validation
All six Secrets exist in the sigma1 namespace; each Secret contains the documented keys; 'kubectl get secret <name> -o jsonpath' returns non-empty values for all keys.