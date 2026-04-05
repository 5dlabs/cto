Implement subtask 1008: Create aggregated 'sigma1-infra-endpoints' ConfigMap

## Objective
Create the 'sigma1-infra-endpoints' ConfigMap that aggregates all service connection strings and API URLs, and make it available across all namespaces.

## Steps
1. Create a ConfigMap 'sigma1-infra-endpoints' in the 'sigma1' namespace containing all endpoint data: POSTGRES_URL (from CloudNative-PG service), REDIS_URL (from Redis/Valkey service), S3_ENDPOINT + S3_PRODUCT_BUCKET + S3_EVENT_BUCKET, SIGNAL_CLI_URL (from Signal-CLI service), CLOUDFLARE_TUNNEL_URL, ELEVENLABS_API_URL, TWILIO_API_URL, STRIPE_API_URL, OPENCORPORATES_API_URL. 2. For cross-namespace access, either: (a) replicate the ConfigMap into each namespace (databases, openclaw, social, web) using a script or tool like kubed/reflector, or (b) use fully-qualified service DNS names so the ConfigMap only needs to exist once. 3. Document all key names and their expected formats. 4. Create a validation script that reads the ConfigMap and attempts a basic connection check to each endpoint.

## Validation
ConfigMap exists and contains all documented keys with non-empty values; a validation pod can mount the ConfigMap via envFrom and successfully resolve/connect to PostgreSQL, Redis, S3, and Signal-CLI endpoints.