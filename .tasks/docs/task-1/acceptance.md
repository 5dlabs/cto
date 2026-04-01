## Acceptance Criteria

- [ ] 1. `kubectl get namespace sigma1-dev` returns Active. 2. `kubectl get secret -n sigma1-dev` lists all four secrets. 3. `kubectl get configmap sigma1-infra-endpoints -n sigma1-dev -o json` contains all five expected keys. 4. A curl pod in the namespace can resolve PM_SERVER_URL and reach LINEAR_API_BASE with a 200/401 (auth expected). 5. Network policy audit confirms egress only to allowed CIDRs.

## Verification Notes

- [ ] Confirm dependencies are satisfied before implementation.
- [ ] Update tests, docs, and configuration touched by this task.
- [ ] Validate the final behavior against the task objective.