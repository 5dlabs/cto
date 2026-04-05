Implement subtask 1006: Create sigma1-infra-endpoints ConfigMap across all namespaces

## Objective
Create the `sigma1-infra-endpoints` ConfigMap in each service namespace, aggregating connection strings for PostgreSQL, Redis, S3, Signal-CLI, and all other infrastructure endpoints.

## Steps
1. Gather all endpoint values from completed infrastructure deployments:
   - POSTGRES_URL: from CloudNative-PG cluster secret
   - REDIS_URL: from Redis operator secret
   - S3_URL, S3_BUCKET_PRODUCTS, S3_BUCKET_EVENTS: from S3 provisioning
   - SIGNAL_CLI_URL: from Signal-CLI service
2. Create a ConfigMap template `sigma1-infra-endpoints` containing all these keys.
3. Apply this ConfigMap to every service namespace: sigma1, openclaw, social, web, databases.
4. Use `kubectl apply -f` with namespace overrides or a Kustomize overlay to deploy across namespaces.
5. Verify each namespace has the ConfigMap with `kubectl -n <ns> get configmap sigma1-infra-endpoints -o yaml`.
6. Ensure downstream services can reference it via `envFrom: [configMapRef: {name: sigma1-infra-endpoints}]`.

## Validation
For each namespace (sigma1, openclaw, social, web, databases), verify the ConfigMap `sigma1-infra-endpoints` exists and contains all expected keys (POSTGRES_URL, REDIS_URL, S3_URL, SIGNAL_CLI_URL, bucket names). Validate that the values are non-empty and well-formed URLs/connection strings.