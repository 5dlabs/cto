Implement subtask 9002: Scale Redis to HA with Sentinel or cluster mode

## Objective
Configure the Redis operator CR to enable high availability with automatic failover using Redis Sentinel or cluster mode. Ensure existing services can connect through the HA-aware endpoint.

## Steps
1. Update the Redis operator CR to enable Sentinel mode with 3 Redis instances and 3 Sentinel instances.
2. Configure Sentinel quorum to 2 for failover decisions.
3. Set `down-after-milliseconds` and `failover-timeout` to production-appropriate values.
4. Apply pod anti-affinity rules so Redis and Sentinel pods are distributed across nodes.
5. Update the Redis connection endpoint in `{project}-infra-endpoints` ConfigMap to point to the Sentinel-aware service (e.g., `redis-sentinel` service).
6. Ensure application services use Sentinel-aware Redis clients (connection string format: `redis-sentinel://<sentinel-host>:<port>/<master-name>`).

## Validation
Verify 3 Redis pods and 3 Sentinel pods are running. Kill the Redis master pod and confirm Sentinel promotes a replica within the configured timeout. Validate application services reconnect without errors. Run `SENTINEL masters` and confirm the new master is recognized.