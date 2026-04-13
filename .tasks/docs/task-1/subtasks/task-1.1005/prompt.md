<identity>
You are bolt working on subtask 1005 of task 1.
</identity>

<context>
<scope>
Define Helm secret templates (or ExternalSecrets if using ESO) for all nine external API secrets required by the platform in the sigma1 namespace.
</scope>
</context>

<implementation_plan>
Create sigma1/infra/templates/secrets.yaml. Use Helm's --set or values file with base64-encoded placeholders; document that real values must be injected via CI/CD or a sealed-secrets workflow before production use. Create the following Secret resources in namespace sigma1: (1) sigma1-stripe-secret: keys STRIPE_SECRET_KEY, STRIPE_WEBHOOK_SECRET. (2) sigma1-opencorporates-secret: key OPENCORPORATES_API_KEY. (3) sigma1-linkedin-secret: keys LINKEDIN_CLIENT_ID, LINKEDIN_CLIENT_SECRET. (4) sigma1-google-secret: keys GOOGLE_REVIEWS_API_KEY, GOOGLE_CALENDAR_CLIENT_ID, GOOGLE_CALENDAR_CLIENT_SECRET. (5) sigma1-elevenlabs-secret: key ELEVENLABS_API_KEY. (6) sigma1-twilio-secret: keys TWILIO_ACCOUNT_SID, TWILIO_AUTH_TOKEN, TWILIO_PHONE_NUMBER. (7) sigma1-openai-secret: key OPENAI_API_KEY. (8) sigma1-cloudflare-secret: keys CF_R2_ACCESS_KEY_ID, CF_R2_SECRET_ACCESS_KEY, CF_R2_ENDPOINT, CF_R2_BUCKET_NAME. (9) sigma1-jwt-secret: key JWT_SECRET. Set type: Opaque for all. Use helm values placeholders (e.g., {{ .Values.secrets.stripe.secretKey | b64enc }}) with a values-secrets.yaml.example file committed showing required keys.
</implementation_plan>

<validation>
kubectl get secret -n sigma1 lists all 9 secrets by name. kubectl get secret sigma1-stripe-secret -n sigma1 -o jsonpath='{.data}' shows both expected keys. Repeat spot-check for sigma1-cloudflare-secret (4 keys) and sigma1-google-secret (3 keys). kubectl describe secret sigma1-jwt-secret -n sigma1 shows key JWT_SECRET with non-zero bytes.
</validation>