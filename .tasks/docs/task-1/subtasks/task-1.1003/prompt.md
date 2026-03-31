Implement subtask 1003: Deploy Redis operator CRs and credential secrets

## Objective
Deploy single-replica Redis custom resources in both namespaces with connection string stored in namespace-scoped secrets.

## Steps
1. Create a Helm template for the Redis operator CR in `charts/hermes-infra/templates/redis.yaml`.
2. Configure single-replica for both dev and staging environments.
3. Ensure the operator creates or you manually template a secret named `hermes-redis-credentials` containing the Redis connection string (host, port, and password if auth is enabled).
4. Add values for memory limits in `values-dev.yaml` (256Mi) and `values-staging.yaml` (512Mi).
5. Verify the Redis instance is accessible from within the namespace.

## Validation
`kubectl get redis -n hermes-dev` shows a Ready Redis instance. `kubectl get secret hermes-redis-credentials -n hermes-dev` exists with valid connection string. A test pod can `redis-cli PING` and receive `PONG`.