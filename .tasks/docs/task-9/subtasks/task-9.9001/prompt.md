Implement subtask 9001: Scale PostgreSQL to HA multi-replica mode via operator CR

## Objective
Update the PostgreSQL operator custom resource to enable high-availability with multiple replicas, streaming replication, and automatic failover. Configure synchronous replication settings and pod anti-affinity to spread replicas across nodes.

## Steps
1. Edit the PostgreSQL operator CR (e.g., CloudNativePG Cluster or Crunchy PGO) to set `instances: 3` (or appropriate replica count).
2. Configure synchronous replication with at least one synchronous standby.
3. Add pod anti-affinity rules to ensure replicas are scheduled on different nodes/zones.
4. Set appropriate storage class and volume size for production workloads.
5. Configure connection pooling (PgBouncer sidecar or built-in pooler) with production pool sizes.
6. Apply the updated CR and verify all replicas come up healthy.
7. Confirm the operator reports the cluster as ready with all replicas in streaming replication.

## Validation
Verify 3 PostgreSQL pods are running on distinct nodes. Confirm replication lag is near-zero via `pg_stat_replication`. Simulate primary pod deletion and verify automatic failover completes within operator SLA (typically <30s). Confirm application reconnects successfully.