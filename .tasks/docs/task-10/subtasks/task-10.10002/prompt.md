Implement subtask 10002: Configure Redis HA with Sentinel and update ConfigMap connection string

## Objective
Scale Redis to a Sentinel or replicated configuration with 3 nodes. Update the `sigma-1-infra-endpoints` ConfigMap with the Sentinel-aware connection string so consumers use the HA endpoint.

## Steps
1. Check the installed Redis operator capabilities. If it supports a replicated/Sentinel mode (e.g., `spec.kubernetesConfig.replicas` or a `RedisSentinel` CRD), configure it for 3 replicas with Sentinel.
2. If the operator doesn't support Sentinel natively, deploy a Redis Sentinel StatefulSet: 3 Redis nodes + 3 Sentinel nodes using a Helm chart (e.g., `bitnami/redis` with `sentinel.enabled=true`).
3. Configure Sentinel to monitor the master named `sigma-1-redis` with quorum 2.
4. Update the `sigma-1-infra-endpoints` ConfigMap: change `REDIS_URL` to the Sentinel-aware connection string format (e.g., `redis-sentinel://sentinel-0:26379,sentinel-1:26379,sentinel-2:26379/sigma-1-redis` or the operator-provided service).
5. Apply ConfigMap changes and verify dependent pods pick up the new endpoint via `envFrom`.
6. Validate Sentinel is correctly monitoring the master: `redis-cli -h <sentinel-svc> -p 26379 sentinel master sigma-1-redis`.

## Validation
Run `redis-cli -h <sentinel-service> -p 26379 sentinel master sigma-1-redis` and verify it returns a valid master IP with `num-slaves: 2`. Verify the ConfigMap `sigma-1-infra-endpoints` contains the updated `REDIS_URL` with Sentinel connection string. Test failover by killing the master pod and confirming Sentinel promotes a replica within 30 seconds.