Implement subtask 9003: Configure Redis HA with Sentinel mode and 3 replicas

## Objective
Configure the Redis operator CR for production with Sentinel mode, 3 replicas, resource requests/limits, and a PodDisruptionBudget.

## Steps
1. Create the production Redis CR in `hermes-production` namespace with Sentinel mode enabled and 3 replicas.
2. Set resource requests/limits: `requests: {cpu: 250m, memory: 256Mi}`, `limits: {cpu: 500m, memory: 512Mi}` per replica.
3. Configure Sentinel with 3 sentinel instances for quorum-based failover.
4. Create a PodDisruptionBudget: `maxUnavailable: 1` for Redis pods.
5. Enable persistence (AOF or RDB) for data durability.
6. Update `hermes-infra-endpoints` ConfigMap if the Redis service name differs in production.

## Validation
Verify `kubectl get pods -n hermes-production -l app=hermes-redis` (or equivalent label) returns 3 Running Redis pods plus 3 Sentinel pods. Verify Sentinel is tracking the master: `kubectl exec` into a sentinel pod and run `redis-cli -p 26379 SENTINEL masters`. Verify PDB exists: `kubectl get pdb -n hermes-production` shows Redis PDB.