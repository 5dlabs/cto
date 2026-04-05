Implement subtask 1007: Create sigma1-infra-endpoints ConfigMap

## Objective
Create the central ConfigMap that aggregates all infrastructure connection strings and endpoints for consumption by backend services via envFrom.

## Steps
1. Create `sigma1-infra-endpoints.yaml` ConfigMap in namespace `sigma1` with the following data keys:
   - `CNPG_SIGMA1_POSTGRES_URL`: `postgresql://sigma1_user@sigma1-postgres-rw.sigma1.svc.cluster.local:5432/sigma1`
   - `REDIS_SIGMA1_VALKEY_URL`: `redis://sigma1-valkey.sigma1.svc.cluster.local:6379`
   - `R2_SIGMA1_ASSETS_ENDPOINT`: The R2 endpoint URL from subtask 1005
   - `R2_SIGMA1_ASSETS_BUCKET`: `sigma1-assets`
   - Per-service database URLs (using each scoped user):
     - `DATABASE_URL_CATALOG`: `postgresql://sigma1_catalog:<secret-ref>@sigma1-postgres-rw.sigma1.svc.cluster.local:5432/sigma1?options=-c%20search_path%3Dpublic`
     - `DATABASE_URL_RMS`: `postgresql://sigma1_rms:<secret-ref>@sigma1-postgres-rw.sigma1.svc.cluster.local:5432/sigma1?options=-c%20search_path%3Drms`
     - `DATABASE_URL_FINANCE`: similar pattern for finance schema
     - `DATABASE_URL_VETTING`: similar pattern for vetting schema
     - `DATABASE_URL_SOCIAL`: similar pattern for social schema
     - `DATABASE_URL_AUDIT`: similar pattern for audit schema
   Note: Passwords should NOT be in the ConfigMap. Services should mount both the ConfigMap (for hostnames) and the relevant user Secret (for password). Alternatively, construct full URLs in an init container. Document the intended consumption pattern.
2. Apply the ConfigMap.
3. Verify all keys are present and non-empty.

## Validation
`kubectl get configmap sigma1-infra-endpoints -n sigma1 -o json | jq '.data | keys'` shows all expected keys. Each value is non-empty. `kubectl get configmap sigma1-infra-endpoints -n sigma1 -o jsonpath='{.data.CNPG_SIGMA1_POSTGRES_URL}'` returns a valid PostgreSQL connection string. No passwords are stored in the ConfigMap data.