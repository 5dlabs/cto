Implement subtask 9001: Scale CloudNative-PG to 3-instance HA with synchronous replication

## Objective
Update the sigma1-postgres Cluster CR to run 3 instances with synchronous replication enabled, configure PgBouncer connection pooling, set production resource requests/limits, and add a PodDisruptionBudget.

## Steps
1. Edit the `sigma1-postgres` Cluster CR YAML:
   - Set `spec.instances: 3`
   - Add `spec.postgresql.synchronous.method: any`, `spec.postgresql.synchronous.number: 1`
   - Or use `spec.minSyncReplicas: 1` and `spec.maxSyncReplicas: 2` depending on CNPG version
2. Configure PgBouncer pooler:
   - Add `spec.pgbouncer.poolMode: transaction`
   - Set `spec.pgbouncer.parameters.max_client_conn: 100`
   - Set `spec.pgbouncer.parameters.default_pool_size: 25`
3. Set resource limits on each instance:
   - `spec.resources.requests.memory: 1Gi`, `spec.resources.requests.cpu: 500m`
   - `spec.resources.limits.memory: 1Gi`, `spec.resources.limits.cpu: 500m`
4. Ensure PodDisruptionBudget is configured:
   - CNPG manages PDBs automatically, but verify `spec.enablePDB: true` or equivalent
   - Verify maxUnavailable is 1
5. Apply the updated CR and verify all 3 instances come up and synchronous replication is active via `kubectl cnpg status sigma1-postgres`.

## Validation
Verify `kubectl cnpg status sigma1-postgres` shows 3 instances with sync replication active. Kill one replica pod and confirm writes continue within 30 seconds via automatic failover. Verify PgBouncer endpoint is accessible and connection pooling is working by checking pgbouncer stats.