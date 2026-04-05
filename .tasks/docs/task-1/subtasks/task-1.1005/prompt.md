Implement subtask 1005: Deploy Valkey CR via Redis operator

## Objective
Deploy the Valkey 7.2 instance using the existing `redis.redis.opstreelabs.in/v1beta2` operator in the `sigma1-db` namespace.

## Steps
1. Create `Redis` CR YAML named `sigma1-valkey` in `sigma1-db` namespace using API version `redis.redis.opstreelabs.in/v1beta2`:
   - `spec.kubernetesConfig.image: valkey/valkey:7.2-alpine`
   - `spec.kubernetesConfig.resources.limits.memory: 256Mi`
   - `spec.kubernetesConfig.resources.limits.cpu: 250m`
   - `spec.kubernetesConfig.resources.requests.memory: 128Mi`
   - `spec.kubernetesConfig.resources.requests.cpu: 100m`
   - Standalone mode (no cluster, no sentinel for dev)
2. Apply the Redis CR.
3. Wait for the Valkey pod to reach Running state.
4. Verify the service `sigma1-valkey.sigma1-db.svc.cluster.local:6379` is reachable.

## Validation
`kubectl get redis sigma1-valkey -n sigma1-db` shows the CR in a ready state. From a pod in sigma1 namespace: `redis-cli -h sigma1-valkey.sigma1-db.svc.cluster.local PING` returns PONG. `redis-cli INFO server` shows valkey version 7.2.