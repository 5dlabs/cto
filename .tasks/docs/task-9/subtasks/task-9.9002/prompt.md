Implement subtask 9002: Scale Redis to HA multi-replica mode

## Objective
Update the Redis operator CR or Helm values to enable Redis Sentinel or Redis Cluster mode with multiple replicas for automatic failover and high availability.

## Steps
1. Identify the existing Redis CR or Helm release.
2. Enable Sentinel mode with at least 3 Sentinel instances and 1 master + 2 replicas, OR enable Redis Cluster mode with appropriate shard count.
3. Set PodDisruptionBudgets (minAvailable: 2 for data nodes).
4. Configure anti-affinity to spread replicas across nodes.
5. Apply the updated configuration.
6. Verify all Redis pods are running and Sentinel/Cluster topology is healthy via `redis-cli info replication` or `cluster info`.

## Validation
Verify Redis master and replica pods are running (3+ total). Confirm Sentinel quorum is established. Kill the master pod and confirm failover completes within 30 seconds. Verify application connectivity after failover.