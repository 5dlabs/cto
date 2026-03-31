Implement subtask 9002: Configure CloudNative-PG HA cluster with 3 replicas and synchronous replication

## Objective
Update the PostgreSQL Cluster CR for production with 3 replicas (1 primary + 2 read replicas), synchronous replication, automated failover, scheduled backups with WAL archiving, resource limits, and a PodDisruptionBudget.

## Steps
1. Create a production CNPG `Cluster` CR (`hermes-pg`) in `hermes-production` namespace with `instances: 3`.
2. Set `postgresql.synchronous.method: any`, `postgresql.synchronous.number: 1` for synchronous replication with `minSyncReplicas: 1`.
3. Configure automated failover (default in CNPG).
4. Add backup configuration: `spec.backup.barmanObjectStore` pointing to MinIO production bucket with a scheduled base backup (e.g., daily at 2am) and continuous WAL archiving.
5. Set resource requests/limits: `requests: {cpu: 1, memory: 2Gi}`, `limits: {cpu: 2, memory: 4Gi}` per replica.
6. Create a PodDisruptionBudget: `maxUnavailable: 1` targeting CNPG pods.
7. Configure storage: production-appropriate PVC size (e.g., 50Gi) with the cluster's default StorageClass.

## Validation
Verify `kubectl get pods -n hermes-production -l cnpg.io/cluster=hermes-pg` returns 3 Running pods. Kill the primary pod and verify automatic failover completes within 30 seconds by running continuous `SELECT 1` queries. Verify backup schedule is registered: `kubectl get scheduledbackup -n hermes-production`.