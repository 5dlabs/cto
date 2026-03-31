Implement subtask 1008: Validate full infrastructure stack end-to-end

## Objective
Run a comprehensive validation of all provisioned infrastructure components to ensure they are healthy, interconnected, and ready for downstream service deployment. Verify cross-namespace access patterns and document the final state.

## Steps
1. Deploy a temporary validation pod in the `sigma1` namespace with `envFrom` referencing `sigma1-infra-endpoints` ConfigMap and all credential secrets.
2. From the pod, test PostgreSQL connectivity: `psql $POSTGRES_URL -c 'SELECT 1'`. Verify each schema is accessible.
3. Test Redis connectivity: `redis-cli -h $REDIS_HOST -p $REDIS_PORT -a $REDIS_PASSWORD PING`.
4. Test S3 connectivity: `aws s3 ls s3://$S3_BUCKET --endpoint-url $S3_ENDPOINT`.
5. Test Signal-CLI: `curl $SIGNALCLI_URL/v1/about`.
6. Verify all secrets are mountable and contain expected key names.
7. Document the final infrastructure state: list all namespaces, running pods in databases and sigma1 namespaces, ConfigMap contents, and secret names (not values).
8. Clean up the validation pod.
9. Create a brief markdown document `docs/infrastructure-manifest.md` listing all endpoints, secret references, and namespace layout for downstream development teams.

## Validation
All validation commands in the details must pass without errors. The validation pod must successfully connect to every infrastructure component. The infrastructure-manifest.md document must be generated and contain accurate information matching the deployed state. After cleanup, verify no test pods remain running.