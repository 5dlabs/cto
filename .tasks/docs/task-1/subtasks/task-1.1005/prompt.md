Implement subtask 1005: Create sigma1-infra-endpoints ConfigMap

## Objective
Create a ConfigMap named 'sigma1-infra-endpoints' in the 'sigma1' namespace, containing connection strings for the deployed PostgreSQL and Redis/Valkey instances.

## Steps
1. Retrieve the internal service URLs for 'sigma1-postgres' and 'sigma1-valkey'.2. Construct the ConfigMap with keys like `POSTGRES_SIGMA1_POSTGRES_URL` and `REDIS_SIGMA1_VALKEY_URL` and their respective connection strings.3. Apply the ConfigMap to the 'sigma1' namespace.

## Validation
1. Verify the 'sigma1-infra-endpoints' ConfigMap exists in the 'sigma1' namespace.2. Inspect the ConfigMap content using `kubectl get cm sigma1-infra-endpoints -n sigma1 -o yaml` and confirm correct URLs for PostgreSQL and Redis/Valkey.