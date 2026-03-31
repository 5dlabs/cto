Implement subtask 9001: Scale PostgreSQL to HA with multi-replica failover

## Objective
Configure the PostgreSQL operator CR to run multiple replicas with streaming replication and automatic failover enabled. Update the existing PostgreSQL CR to increase replica count, configure replication slots, and set failover policies.

## Steps
1. Update the PostgreSQL operator CR (e.g., CloudNativePG Cluster or Zalando PostgreSQL) to set `instances: 3` (1 primary + 2 replicas).
2. Configure synchronous replication with at least 1 synchronous standby.
3. Set `failoverDelay` and health check intervals appropriate for production.
4. Ensure PVCs use a StorageClass with `reclaimPolicy: Retain` for data safety.
5. Configure pod anti-affinity rules to spread replicas across nodes.
6. Verify the operator's built-in failover promotion logic is enabled.
7. Update connection strings in the `{project}-infra-endpoints` ConfigMap if the service endpoint changes (e.g., to an `-rw` or `-ro` service).

## Validation
Verify 3 PostgreSQL pods are running and healthy. Kill the primary pod and confirm a replica is promoted within the configured failover window. Validate that application services reconnect automatically without data loss. Check `pg_stat_replication` shows active streaming replication.