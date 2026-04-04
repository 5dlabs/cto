## Acceptance Criteria

- [ ] 1. `kubectl get accesstunnel -n sigma-1-dev` (or `clustertunnel`) returns a CR in 'Active' or 'Ready' state. 2. External HTTPS request to the tunnel URL returns a valid response (200 from PM server /health endpoint) with valid TLS certificate. 3. `kubectl auth can-i create secrets --as=system:serviceaccount:sigma-1-dev:sigma-1-pm-server -n sigma-1-dev` returns 'no'. 4. `kubectl auth can-i get configmaps --as=system:serviceaccount:sigma-1-dev:sigma-1-pm-server -n sigma-1-dev` returns 'yes'. 5. All pods in `sigma-1-dev` have resource requests and limits set (verified via `kubectl describe pod` showing non-zero CPU/memory requests). 6. Liveness and readiness probes are configured on all deployments (verified via `kubectl get deployment -o json` showing probe configurations). 7. `kubectl get networkpolicy -n sigma-1-dev` returns at least one NetworkPolicy. 8. PM server pod can reach discord-bridge-http (verified via exec curl), but cannot reach arbitrary external hosts (verified via exec curl to a blocked destination returning connection refused or timeout).

## Verification Notes

- [ ] Confirm dependencies are satisfied before implementation.
- [ ] Update tests, docs, and configuration touched by this task.
- [ ] Validate the final behavior against the task objective.