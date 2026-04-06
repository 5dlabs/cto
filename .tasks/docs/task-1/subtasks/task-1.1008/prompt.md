Implement subtask 1008: Validate end-to-end infrastructure connectivity

## Objective
Run a comprehensive validation suite to ensure all provisioned infrastructure components are reachable, secrets are accessible, and the ConfigMap is correctly populated and consumable by downstream services.

## Steps
1. Deploy a temporary debug pod in the sigma1 namespace with envFrom referencing sigma1-infra-endpoints ConfigMap.
2. From the debug pod, test connectivity to:
   - PostgreSQL: psql to $POSTGRES_HOST and verify schemas exist (\dn)
   - Redis: redis-cli PING to $REDIS_HOST
   - S3/R2: aws s3 ls with credentials from sigma1-s3-credentials secret
   - Signal-CLI: curl $SIGNALCLI_URL/v1/about
3. Verify all external API secrets are mounted and readable.
4. Check that all secrets have non-empty values for their expected keys.
5. Document any issues found and confirm all checks pass.
6. Clean up the debug pod after validation.

## Validation
All connectivity checks from the debug pod succeed: PostgreSQL schemas are queryable, Redis responds to PING, S3 bucket listing returns success, Signal-CLI returns version info. All 6 external API secrets have non-empty values for expected keys. Debug pod is cleaned up after validation.