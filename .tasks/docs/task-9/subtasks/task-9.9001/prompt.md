Implement subtask 9001: Scale PostgreSQL to HA with CloudNative-PG multi-replica and failover

## Objective
Update the CloudNative-PG cluster CR to enable multi-replica (at least 2 replicas + 1 primary), configure streaming replication, automatic failover, and pod anti-affinity rules for production resilience.

## Steps
1. Edit the CloudNative-PG Cluster CR to set `instances: 3` (1 primary, 2 replicas).
2. Configure `podAntiAffinity` to spread instances across nodes/zones.
3. Enable `enableSuperuserAccess: false` for security.
4. Set `postgresql.pg_hba` to restrict replication connections.
5. Configure backup and WAL archiving for point-in-time recovery.
6. Verify the switchover/failover promotion policy (e.g., `failoverDelay`, `switchoverDelay`).
7. Update the `{project}-infra-endpoints` ConfigMap with the HA-aware PostgreSQL service endpoint (the `-rw` service for writes, `-ro` for reads if applicable).
8. Apply the updated CR and confirm all replicas are streaming.

## Validation
Verify 3 PostgreSQL pods are running across different nodes; confirm streaming replication lag < 1s; simulate primary pod deletion and confirm automatic failover completes within 30s; verify application reconnects to new primary without manual intervention.