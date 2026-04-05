Implement subtask 9001: Scale PostgreSQL to HA multi-replica mode

## Objective
Update the PostgreSQL operator CR (e.g., CloudNativePG Cluster or Zalando postgresql) to run multiple replicas with streaming replication, ensuring automatic failover and read replicas are configured for production workloads.

## Steps
1. Identify the existing PostgreSQL CR manifest in the infra repo.
2. Increase `spec.instances` (CloudNativePG) or `numberOfInstances` (Zalando) to at least 3 replicas.
3. Configure synchronous replication for at least one standby to prevent data loss.
4. Ensure PodDisruptionBudgets are set (minAvailable: 2).
5. Verify anti-affinity rules so replicas land on different nodes.
6. Apply the updated CR and confirm all replicas reach streaming state via `kubectl get pods` and operator status.

## Validation
Verify 3 PostgreSQL pods are running. Confirm replication lag is near-zero via `pg_stat_replication`. Kill the primary pod and confirm automatic failover completes within 30 seconds with no data loss.