## Acceptance Criteria

- [ ] 1. `kubectl get hpa -n sigma1-prod` shows PM server HPA with correct min/max/target. 2. `kubectl get ingress -n sigma1-prod` shows both ingresses with TLS configured. 3. `curl -I https://api.sigma1.5dlabs.io/health` returns 200 with valid TLS certificate. 4. `curl -I https://sigma1.5dlabs.io` returns 200 with Cache-Control header on static assets. 5. PDB validation: `kubectl get pdb -n sigma1-prod` shows correct minAvailable values. 6. Kill one PM server pod; verify traffic continues to be served (zero downtime confirmed by continuous health check). 7. Pod anti-affinity: verify pods are distributed across at least 2 nodes/zones via `kubectl get pods -o wide`.

## Verification Notes

- [ ] Confirm dependencies are satisfied before implementation.
- [ ] Update tests, docs, and configuration touched by this task.
- [ ] Validate the final behavior against the task objective.