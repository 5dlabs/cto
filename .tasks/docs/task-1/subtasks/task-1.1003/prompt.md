Implement subtask 1003: Deploy Redis/Valkey using Opstree operator

## Objective
Deploy a single-instance Redis/Valkey cache using the Opstree Redis operator in the databases namespace, configured with 7.2-alpine image.

## Steps
1. Ensure the Opstree Redis operator is installed in the cluster (install via Helm if not present). 2. Write a Redis CR YAML: standalone mode, single replica, image tag 7.2-alpine, in 'databases' namespace. 3. Configure persistence if needed (small PVC for AOF/RDB). 4. Set resource requests/limits for dev (128Mi-256Mi RAM, 250m CPU). 5. Optionally set a requirepass via a Kubernetes Secret. 6. Apply the CR and wait for the Redis pod to become Ready. 7. Record the Redis connection URL (redis://<host>:<port>) for the aggregated ConfigMap.

## Validation
Redis pod is Running and Ready; connect via redis-cli PING and receive PONG; verify SET/GET operations succeed.