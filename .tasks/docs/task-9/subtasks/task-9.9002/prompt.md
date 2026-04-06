Implement subtask 9002: Scale Redis/Valkey to HA mode

## Objective
Update the Redis/Valkey deployment to run in HA mode with sentinel or replication, configure automatic failover, and validate session/cache continuity during failover.

## Steps
1. Update the Redis/Valkey CR or Helm values to enable HA with at least 1 primary and 2 replicas plus sentinel instances (or use Redis Cluster mode depending on operator). 2. Configure PodDisruptionBudgets (minAvailable for quorum). 3. Set anti-affinity rules to spread across nodes. 4. Update the `{project}-infra-endpoints` ConfigMap with the sentinel/cluster endpoint so application services connect to the HA endpoint rather than a single instance. 5. Apply and verify all Redis pods reach Ready state. 6. Confirm sentinel correctly identifies master and replicas.

## Validation
Verify Redis sentinel/cluster pods are running. Kill the primary Redis pod and confirm sentinel promotes a replica within the expected timeout. Verify application services reconnect transparently. Confirm no cache data loss for persistent keys.