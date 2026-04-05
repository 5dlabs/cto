Implement subtask 1003: Deploy Redis/Valkey via opstreelabs operator

## Objective
Deploy a single-replica Redis/Valkey instance using the opstreelabs Redis operator in the databases namespace for caching, rate limiting, and session storage.

## Steps
1. Ensure the opstreelabs Redis operator is installed (check CRD: `kubectl get crds | grep redis.redis.opstreelabs.in`).
2. Create a `Redis` CR YAML in the `databases` namespace specifying single-replica mode, resource limits, persistence volume.
3. Configure password authentication via a Kubernetes Secret referenced by the CR.
4. Apply the CR: `kubectl apply -f redis-cluster.yaml`.
5. Wait for the Redis pod to be Running and Ready.
6. Test connectivity: `kubectl -n databases exec -it <redis-pod> -- redis-cli ping` should return PONG.
7. Record the REDIS_URL (host, port, password) for ConfigMap creation.

## Validation
Confirm the Redis CR is in Ready state. Execute `redis-cli ping` from within the pod and verify PONG response. Confirm the password secret exists and matches the CR reference.