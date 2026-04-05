Implement subtask 1003: Deploy Redis/Valkey via Opstree operator

## Objective
Deploy a single-instance Redis/Valkey instance using the Opstree Redis operator in the databases namespace for caching, rate limiting, and session storage.

## Steps
1. Create a Redis CR (redis.redis.opstreelabs.in/v1beta2) in the databases namespace.
2. Configure as single-replica standalone mode with appropriate resource limits.
3. Optionally set a password via Kubernetes Secret.
4. Apply the CR and wait for the Redis pod to reach Running status.
5. Verify connectivity with redis-cli PING from a test pod.
6. Record the REDIS_URL (host:port and optional auth) for later ConfigMap aggregation.

## Validation
Verify the Redis pod is Running. From a test pod, run `redis-cli -h <host> -p <port> PING` and confirm PONG response. Verify the Redis CR status reports healthy.