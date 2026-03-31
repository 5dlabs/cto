Implement subtask 1006: Create hermes-infra-endpoints ConfigMap aggregating all service connection strings

## Objective
Create the hermes-infra-endpoints ConfigMap in each namespace that aggregates all infrastructure connection strings for downstream service consumption via envFrom.

## Steps
1. Create Helm template `charts/hermes-infra/templates/configmap-endpoints.yaml`.
2. Populate the ConfigMap with the following keys, referencing values from the deployed services:
   - `CNPG_HERMES_URL` — PostgreSQL connection string (composed from `hermes-pg-credentials` secret values or operator-generated service DNS)
   - `REDIS_HERMES_URL` — Redis connection string (from `hermes-redis-credentials`)
   - `NATS_HERMES_URL` — NATS connection string (from `hermes-nats-credentials`)
   - `MINIO_HERMES_ENDPOINT` — MinIO S3 endpoint URL
   - `MINIO_HERMES_BUCKET` — Bucket name (`hermes-artifacts-dev` or `hermes-artifacts-staging`)
   - `ENVIRONMENT` — `dev` or `staging` (from values)
3. Note: Sensitive values (passwords, keys) remain in secrets. The ConfigMap contains only hostnames/URLs/bucket names. Downstream pods mount BOTH the ConfigMap (via `envFrom: configMapRef`) and secrets (via `envFrom: secretRef`).
4. Document the expected env var contract in a comment block within the template.
5. Deploy a validation test pod that mounts the ConfigMap and all four secrets, then connects to PostgreSQL, Redis, NATS, and MinIO to verify all four connections succeed.

## Validation
`kubectl get configmap hermes-infra-endpoints -n hermes-dev -o jsonpath='{.data.CNPG_HERMES_URL}'` returns a valid PostgreSQL connection string. A test pod using only `envFrom` from the ConfigMap and secrets successfully connects to all four services.