## Acceptance Criteria

- [ ] 1. RBAC verification: exec into a PM server pod and attempt to list pods — should receive 403 Forbidden. 2. RBAC verification: exec into a PM server pod and read `sigma1-infra-endpoints` ConfigMap — should succeed. 3. ServiceAccount audit: `kubectl get pods -n sigma1-prod -o jsonpath='{.items[*].spec.serviceAccountName}'` returns only `sa-pm-server` and `sa-frontend`, never `default`. 4. Secret rotation: manually trigger rotation CronJob; verify new secret value is mounted in pods after rolling restart (check pod restart timestamp). 5. Audit logging: perform a kubectl create in sigma1-prod; verify the action appears in audit log within 60 seconds. 6. NetworkPolicy: from a frontend pod, attempt to curl an external API directly — should be blocked; curl PM server on port 3000 — should succeed. 7. `docs/security.md` exists and contains RBAC role descriptions and rotation schedule table.

## Verification Notes

- [ ] Confirm dependencies are satisfied before implementation.
- [ ] Update tests, docs, and configuration touched by this task.
- [ ] Validate the final behavior against the task objective.