## Acceptance Criteria

- [ ] 1. `kubectl get pods -n sigma-1 -l app=cto-pm` shows >= 2 running pods distributed across different nodes. 2. PodDisruptionBudget allows voluntary disruption only when >= 1 pod remains: `kubectl get pdb -n sigma-1` shows minAvailable=1. 3. NetworkPolicy is applied: `kubectl get networkpolicy -n sigma-1` lists at least one policy with default-deny and explicit allow rules. 4. Test network policy enforcement: a pod without allowed labels in sigma-1 cannot reach cto-pm (connection timeout). 5. Ingress returns valid TLS certificate: `curl -v https://{ingress-host}` shows TLS handshake with valid cert. 6. HPA is configured: `kubectl get hpa -n sigma-1` shows cto-pm HPA with min=2, max=5, target CPU=70%. 7. All pods have resource requests and limits set: `kubectl describe pod -n sigma-1` shows non-zero values for requests.cpu, requests.memory, limits.cpu, limits.memory.

## Verification Notes

- [ ] Confirm dependencies are satisfied before implementation.
- [ ] Update tests, docs, and configuration touched by this task.
- [ ] Validate the final behavior against the task objective.