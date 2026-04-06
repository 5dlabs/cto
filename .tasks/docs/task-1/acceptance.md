## Acceptance Criteria

- [ ] 1. kubectl get pods -n databases shows sigma1-postgres-1 and sigma1-valkey-0 in Running state within 5 minutes of apply. 2. psql -U sigma1_user -d sigma1 -c '\dn' returns catalog, rms, finance, vetting, social, audit schemas. 3. redis-cli -u $VALKEY_URL ping returns PONG. 4. kubectl get secret -n sigma1 shows all 9 secrets present. 5. kubectl get pod -n signal shows signal-cli pod Running. 6. kubectl get configmap sigma1-infra-endpoints -n sigma1 -o yaml contains all 5 required keys. 7. ArgoCD application shows Synced and Healthy.

## Verification Notes

- [ ] Confirm dependencies are satisfied before implementation.
- [ ] Update tests, docs, and configuration touched by this task.
- [ ] Validate the final behavior against the task objective.