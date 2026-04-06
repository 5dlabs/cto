Implement subtask 1003: Deploy Redis/Valkey instance via operator

## Objective
Deploy a single-replica Redis-compatible instance (Valkey v7.2-alpine) using the Redis operator in the databases namespace.

## Steps
1. Write a Redis CR manifest targeting the redis.redis.opstreelabs.in operator (or chosen Valkey operator).
2. Configure single replica, v7.2-alpine image, persistence enabled with a small PVC (e.g., 5Gi).
3. Set resource requests/limits for dev (128Mi-256Mi RAM, 100m-250m CPU).
4. Configure a password secret for Redis AUTH.
5. Apply the CR to the 'databases' namespace.
6. Wait for the Redis pod to reach Running and Ready.
7. Record the Redis connection string (host:port, password secret ref) for the ConfigMap.

## Validation
Confirm the Redis/Valkey pod is Running; exec into a test pod and run `redis-cli -h <host> -a <password> PING` and verify 'PONG' response; confirm the password secret exists in the databases namespace.