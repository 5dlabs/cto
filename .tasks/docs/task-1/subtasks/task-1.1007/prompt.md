Implement subtask 1007: Assemble sigma1-infra-endpoints ConfigMap

## Objective
Create the sigma1-infra-endpoints ConfigMap aggregating all connection strings, service URLs, bucket names, and secret references from all provisioned infrastructure components.

## Steps
1. Create a ConfigMap named 'sigma1-infra-endpoints' in the sigma1 namespace with the following keys:
   - POSTGRES_HOST, POSTGRES_PORT, POSTGRES_DB, POSTGRES_USER_SECRET_REF
   - REDIS_HOST, REDIS_PORT, REDIS_PASSWORD_SECRET_REF
   - S3_ENDPOINT, S3_PRODUCT_IMAGES_BUCKET, S3_EVENT_PHOTOS_BUCKET, S3_CREDENTIALS_SECRET_REF
   - SIGNAL_CLI_URL (internal service URL)
   - ELEVENLABS_SECRET_REF, TWILIO_SECRET_REF, STRIPE_SECRET_REF, OPENCORPORATES_SECRET_REF, LINKEDIN_SECRET_REF, GOOGLE_REVIEWS_SECRET_REF, CREDIT_API_SECRET_REF
2. Use the actual values gathered from subtasks 1002-1006.
3. Apply the ConfigMap.
4. Verify all keys are populated and non-empty.

## Validation
Run `kubectl get configmap sigma1-infra-endpoints -n sigma1 -o yaml` and verify all expected keys are present and non-empty; confirm the ConfigMap has at least 15 key-value entries covering all infrastructure components.