Implement subtask 1008: Validate end-to-end infrastructure connectivity from all namespaces

## Objective
Run connectivity tests from each service namespace to verify that all infrastructure components (PostgreSQL, Redis, S3, Signal-CLI) are reachable using the ConfigMap and Secret values.

## Steps
1. Deploy a temporary debug pod (e.g., `curlimages/curl` or `busybox` with psql/redis-cli) in each service namespace (sigma1, openclaw, social, web).
2. From each pod, test:
   - PostgreSQL: `psql $POSTGRES_URL -c 'SELECT 1'`
   - Redis: `redis-cli -u $REDIS_URL ping`
   - S3: `curl $S3_URL` or attempt a ListBuckets call with credentials
   - Signal-CLI: `curl $SIGNAL_CLI_URL/v1/about`
3. Verify all connections succeed from every namespace.
4. Check that secrets are mountable and contain non-empty values.
5. Clean up debug pods after verification.
6. Document results and any connectivity issues.

## Validation
All connectivity tests pass from every service namespace: PostgreSQL returns SELECT 1, Redis returns PONG, S3 endpoint responds, Signal-CLI returns a valid /about response. Zero connection failures across all namespaces.