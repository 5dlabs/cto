Implement subtask 9002: Scale Redis/Valkey to HA with replica and sentinel/cluster configuration

## Objective
Update the Redis/Valkey operator CR to enable HA mode with multiple replicas and sentinel-based or cluster-based failover for production cache resilience.

## Steps
1. Update the Valkey (or Redis) operator CR to configure at least 3 nodes (1 primary + 2 replicas) with sentinel mode enabled.
2. Configure sentinel to monitor the primary and trigger automatic failover.
3. Set resource requests/limits appropriate for production workloads.
4. Configure `podAntiAffinity` to distribute replicas across nodes.
5. Enable persistence (AOF or RDB) if required for session durability.
6. Update the `{project}-infra-endpoints` ConfigMap with the sentinel-aware Redis endpoint (sentinel service address and port).
7. Apply and verify all replicas are connected and sentinel is monitoring.

## Validation
Verify 3 Redis/Valkey pods running with sentinel; simulate primary pod kill and confirm sentinel promotes a replica within 15s; verify client libraries reconnect via sentinel; confirm no data loss for persisted keys.