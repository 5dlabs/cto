Implement subtask 1007: Create sigma1-infra-endpoints ConfigMap aggregating all service endpoints

## Objective
Create the central ConfigMap named sigma1-infra-endpoints in the sigma1 namespace, containing connection strings and API URLs for all provisioned services.

## Steps
1. Create a ConfigMap manifest named sigma1-infra-endpoints in the sigma1 namespace.
2. Include the following keys with values from prior subtasks:
   - POSTGRES_URL: connection string from CloudNative-PG deployment
   - REDIS_URL: connection string from Redis/Valkey deployment
   - S3_ENDPOINT: S3/R2 endpoint URL
   - S3_PRODUCT_IMAGES_BUCKET: bucket name
   - S3_EVENT_PHOTOS_BUCKET: bucket name
   - SIGNAL_CLI_URL: http://signal-cli.openclaw.svc.cluster.local:8080
   - ELEVENLABS_SECRET_REF: sigma1-elevenlabs-secret
   - TWILIO_SECRET_REF: sigma1-twilio-secret
   - STRIPE_SECRET_REF: sigma1-stripe-secret
   - OPENCORPORATES_SECRET_REF: sigma1-opencorporates-secret
   - LINKEDIN_SECRET_REF: sigma1-linkedin-secret
   - GOOGLE_REVIEWS_SECRET_REF: sigma1-google-reviews-secret
3. Apply the ConfigMap.
4. Also copy or mirror the ConfigMap into other namespaces (databases, openclaw, social, web) if cross-namespace access is needed, or document how services should reference it.

## Validation
Verify the ConfigMap exists in the sigma1 namespace with all expected keys. From a test pod using envFrom referencing the ConfigMap, echo each environment variable and confirm non-empty values. Verify POSTGRES_URL connects to PostgreSQL, REDIS_URL connects to Redis, and SIGNAL_CLI_URL returns a valid response.