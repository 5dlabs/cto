Implement subtask 9001: Create hermes-production namespace with ConfigMap and Secret patterns

## Objective
Create the hermes-production namespace with appropriate labels and annotations, and replicate all ConfigMap and Secret patterns from the dev namespace (Task 1) with production-specific values including the hermes-infra-endpoints ConfigMap.

## Steps
1. Create `hermes-production` namespace YAML with labels: `app.kubernetes.io/part-of: hermes`, `environment: production`.
2. Copy and adapt the `hermes-infra-endpoints` ConfigMap from the dev setup (Task 1), updating all service endpoints to point to production-suffixed services (e.g., `hermes-pg-rw.hermes-production.svc`).
3. Create production Secret objects for PostgreSQL, Redis, NATS, and MinIO credentials (placeholder values to be replaced by secret rotation in Task 10).
4. Ensure all resources are namespaced to `hermes-production`.
5. Apply resource quotas on the namespace to prevent runaway resource consumption.

## Validation
Verify `kubectl get ns hermes-production` returns the namespace with correct labels. Verify `kubectl get configmap hermes-infra-endpoints -n hermes-production` returns the ConfigMap with production endpoint values. Verify `kubectl get secrets -n hermes-production` lists all expected credential secrets.