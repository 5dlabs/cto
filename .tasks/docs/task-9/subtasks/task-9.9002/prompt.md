Implement subtask 9002: Scale Redis/Valkey to HA multi-replica mode

## Objective
Update the Redis/Valkey operator CR to enable HA with sentinel or cluster mode, providing automatic failover for the production cache layer.

## Steps
1. Update the Redis operator CR to enable sentinel-based HA with at least 3 sentinel instances and 1 primary + 2 replicas. 2. Configure `redis.replicas: 2` and `sentinel.replicas: 3`. 3. Set anti-affinity rules to distribute pods across nodes. 4. Update resource requests/limits for production workload. 5. Ensure the connection string in the `{project}-infra-endpoints` ConfigMap is updated to point to the sentinel endpoint. 6. Apply the CR and verify sentinel quorum is established. 7. Confirm all application services using Redis reconnect via the sentinel endpoint.

## Validation
Verify sentinel quorum with `redis-cli -p 26379 sentinel masters`. Kill the Redis primary pod and confirm sentinel promotes a replica within 15 seconds. Verify application services continue operating without errors after failover.