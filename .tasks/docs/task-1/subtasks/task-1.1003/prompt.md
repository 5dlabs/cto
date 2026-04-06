Implement subtask 1003: Deploy Redis/Valkey via Opstree operator

## Objective
Deploy a single-instance Redis/Valkey cache using the Opstree Redis operator in the databases namespace.

## Steps
1. Create a Redis CR manifest for the Opstree operator (redis.redis.opstreelabs.in/v1beta2):
   - name: sigma1-redis
   - namespace: databases
   - mode: standalone (single instance for dev)
   - resources: requests 256Mi memory, limits 512Mi
   - persistence: enabled, 10Gi
2. Apply the Redis CR.
3. Wait for the redis pod to reach Running status.
4. Verify the operator creates the service sigma1-redis.databases.svc.cluster.local:6379.
5. If the operator generates an auth secret, note it for ConfigMap. If not, create a password secret sigma1-redis-auth.
6. Record the connection URL: redis://:password@sigma1-redis.databases.svc.cluster.local:6379 for the ConfigMap.

## Validation
Verify the Redis pod is Running in databases namespace. Use `redis-cli -h sigma1-redis.databases.svc.cluster.local PING` from a debug pod and confirm PONG response. Verify the auth secret exists.