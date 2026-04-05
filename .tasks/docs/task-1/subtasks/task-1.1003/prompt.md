Implement subtask 1003: Deploy Redis/Valkey via opstreelabs operator

## Objective
Deploy a single-replica Redis/Valkey instance in the databases namespace using the opstreelabs Redis operator, configured for caching, rate limiting, and session storage.

## Steps
1. Verify the opstreelabs Redis operator (redis.redis.opstreelabs.in) is installed in-cluster; if not, add it as a Helm dependency.
2. Author a Redis CR in infra/redis/redis.yaml: single replica (standalone mode), namespace 'databases', resource requests (256Mi RAM / 250m CPU), persistence enabled with a small PVC (1Gi).
3. Set a password via a Kubernetes Secret 'redis-auth' in the databases namespace.
4. Configure maxmemory-policy as 'allkeys-lru' for cache-friendly behavior.
5. Expose the Redis service as a ClusterIP service 'redis-sigma1' on port 6379.
6. Apply and wait for the pod to reach Running/Ready.

## Validation
kubectl get pods -n databases shows redis pod Running and Ready; exec into a test pod and run 'redis-cli -h redis-sigma1.databases.svc -a <password> PING' and confirm 'PONG'; verify the Secret 'redis-auth' exists.