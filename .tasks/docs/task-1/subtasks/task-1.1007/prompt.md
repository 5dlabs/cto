Implement subtask 1007: Create sigma1-infra-endpoints ConfigMap aggregating all connection strings

## Objective
Create the central ConfigMap 'sigma1-infra-endpoints' in the sigma1 namespace that aggregates connection strings and endpoint URLs for all provisioned infrastructure (PostgreSQL, Redis, S3, Signal-CLI). All downstream services will reference this ConfigMap via envFrom.

## Steps
1. Create a ConfigMap named `sigma1-infra-endpoints` in the `sigma1` namespace.
2. Populate with the following keys (values derived from deployed services):
   - `POSTGRES_HOST`: `sigma1-pg-rw.databases.svc.cluster.local`
   - `POSTGRES_PORT`: `5432`
   - `POSTGRES_DB`: `app` (or the default DB name from CNPG)
   - `POSTGRES_URL`: Full connection string (without password — password comes from the CNPG secret)
   - `REDIS_HOST`: `sigma1-redis.databases.svc.cluster.local`
   - `REDIS_PORT`: `6379`
   - `REDIS_URL`: `redis://:$(REDIS_PASSWORD)@sigma1-redis.databases.svc.cluster.local:6379`
   - `S3_ENDPOINT`: From the S3/R2 provisioning step
   - `S3_BUCKET`: `sigma1-assets`
   - `SIGNALCLI_URL`: `http://sigma1-signal-cli.sigma1.svc.cluster.local:8080`
3. Also copy or reference this ConfigMap into other namespaces (openclaw, social, web) that need it, or document that services should reference it cross-namespace.
4. Apply the ConfigMap manifest.
5. Verify all values resolve to reachable endpoints from within the cluster.

## Validation
Verify the ConfigMap exists with `kubectl get configmap sigma1-infra-endpoints -n sigma1 -o yaml`. Confirm all expected keys are present (POSTGRES_HOST, POSTGRES_PORT, POSTGRES_URL, REDIS_HOST, REDIS_PORT, REDIS_URL, S3_ENDPOINT, S3_BUCKET, SIGNALCLI_URL). Deploy a test pod with `envFrom` referencing the ConfigMap and verify all environment variables are set. From the test pod, attempt a TCP connection to each host:port to confirm reachability.