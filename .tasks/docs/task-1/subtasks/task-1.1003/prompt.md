Implement subtask 1003: Deploy Redis single-replica instance

## Objective
Deploy a single-replica Redis instance named `sigma-1-redis` in the `sigma-1-dev` namespace with no persistence, and expose it via a ClusterIP Service.

## Steps
1. Create a Deployment manifest: name=`sigma-1-redis`, namespace=`sigma-1-dev`, image=`redis:7-alpine`, replicas=1, no persistent volume (data is ephemeral for validation).
2. Set resource requests (cpu=100m, memory=128Mi) and limits (cpu=250m, memory=256Mi) to stay within namespace quota.
3. Configure a readiness probe: `redis-cli ping` on port 6379.
4. Create a ClusterIP Service `sigma-1-redis` on port 6379 targeting the Redis pod.
5. Apply both manifests.
6. Wait for the Deployment to have 1/1 ready replicas.
7. Record the service URL `redis://sigma-1-redis.sigma-1-dev.svc.cluster.local:6379` for the ConfigMap.

## Validation
`kubectl get deployment sigma-1-redis -n sigma-1-dev` shows 1/1 READY. `kubectl get svc sigma-1-redis -n sigma-1-dev` shows ClusterIP on port 6379. A transient pod running `redis-cli -h sigma-1-redis.sigma-1-dev.svc.cluster.local ping` returns PONG.