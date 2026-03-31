## Acceptance Criteria

- [ ] 1. `kubectl get namespace hermes-dev hermes-staging` returns both namespaces in Active state.
- [ ] 2. `kubectl get clusters.postgresql.cnpg.io -n hermes-dev` shows a Ready cluster with 1 replica.
- [ ] 3. `kubectl get configmap hermes-infra-endpoints -n hermes-dev -o jsonpath='{.data.CNPG_HERMES_URL}'` returns a valid PostgreSQL connection string.
- [ ] 4. `kubectl exec` a test pod in `hermes-dev` that connects to PostgreSQL, Redis, NATS, and MinIO using only env vars from `hermes-infra-endpoints` ConfigMap and mounted secrets — all four connections succeed.
- [ ] 5. MinIO bucket `hermes-artifacts-dev` exists and is writable with dedicated credentials, AND those credentials do NOT have access to any GitLab-owned buckets.
- [ ] 6. ArgoCD UI shows `hermes-backend-dev` and `hermes-frontend-staging` Application CRs in Synced/Healthy state (initially empty).
- [ ] 7. A structured JSON log emitted from a test pod in `hermes-dev` is queryable in Loki via LogQL within 30 seconds.

## Verification Notes

- [ ] Confirm dependencies are satisfied before implementation.
- [ ] Update tests, docs, and configuration touched by this task.
- [ ] Validate the final behavior against the task objective.