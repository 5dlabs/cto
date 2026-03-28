Implement subtask 1003: Deploy Redis/Valkey operator and instance

## Objective
Deploy the Redis/Valkey operator and provision a single-instance Redis/Valkey instance named 'sigma1-valkey' (Valkey 7.2-alpine) within the 'databases' namespace.

## Steps
1. Install Redis/Valkey operator using Helm or kubectl apply.2. Create a `Redis` or `Valkey` custom resource for 'sigma1-valkey' in the 'databases' namespace, specifying Valkey 7.2-alpine and single instance configuration.

## Validation
1. Verify Redis/Valkey operator pods are running in the 'databases' namespace.2. Confirm 'sigma1-valkey' instance is running and accessible via `redis-cli` from within the cluster.