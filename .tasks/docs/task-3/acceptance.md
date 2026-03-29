## Acceptance Criteria

- [ ] 1. `helm template` with `values-prod.yaml` renders without errors and produces: 3-replica CloudNativePG cluster, HPA with min=2/max=10, at least 4 NetworkPolicy resources, an Ingress with TLS block, a ResourceQuota, and a ServiceAccount. 2. `kubectl apply --dry-run=server` of all production manifests succeeds against a test cluster API. 3. NetworkPolicy validation: a test pod in the notifycore namespace can reach notifycore-pg on 5432 and notifycore-redis on 6379, but cannot reach external IPs or other namespaces. A pod outside the namespace cannot reach notifycore pods on 8080 (only the ingress controller can). 4. CI pipeline definition file exists and contains stages for lint, test, build, and deploy with the sqlx prepare check step present. 5. PodDisruptionBudget allows at most 1 unavailable pod for both the app and database deployments.

## Verification Notes

- [ ] Confirm dependencies are satisfied before implementation.
- [ ] Update tests, docs, and configuration touched by this task.
- [ ] Validate the final behavior against the task objective.