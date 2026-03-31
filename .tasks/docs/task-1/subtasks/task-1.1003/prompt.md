Implement subtask 1003: Deploy Redis/Valkey instance via Opstree operator

## Objective
Deploy a single-instance Redis/Valkey cache using the Opstree Redis operator in the databases namespace. Configure it for session storage, caching, and rate limiting use cases across Sigma-1 services.

## Steps
1. Verify the Opstree Redis operator is installed: `kubectl get crd redis.redis.opstreelabs.in`.
2. Create a `Redis` CR YAML in the `databases` namespace: name `sigma1-redis`, single replica (no cluster mode for v1), resource requests of 256Mi memory / 100m CPU.
3. Configure persistence with a 10Gi PVC for data durability across restarts.
4. Set `maxmemory-policy` to `allkeys-lru` for cache eviction.
5. Enable password authentication via a Kubernetes Secret referenced in the CR.
6. Apply the CR: `kubectl apply -f sigma1-redis.yaml`.
7. Verify the Redis pod is Running and the service endpoint is reachable.
8. Record the service DNS name (e.g., `sigma1-redis.databases.svc.cluster.local:6379`) for the ConfigMap.

## Validation
Verify the Redis pod is Running in the databases namespace. Port-forward to the Redis service and run `redis-cli ping` expecting `PONG`. Authenticate with the password from the secret. Run `SET testkey testvalue` and `GET testkey` to confirm read/write. Check `INFO server` to verify single-instance mode.