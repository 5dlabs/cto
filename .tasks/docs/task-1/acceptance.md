## Acceptance Criteria

- [ ] 1. `kubectl get cluster sigma1-postgres -n sigma1 -o jsonpath='{.status.phase}'` returns `Cluster in healthy state`. 2. `psql` connection using each per-service credential succeeds and can CREATE TABLE in its own schema but gets permission denied on other schemas. 3. `redis-cli -u $REDIS_SIGMA1_VALKEY_URL PING` returns PONG. 4. ConfigMap `sigma1-infra-endpoints` contains all 4+ keys with non-empty values. 5. ExternalSecret CRs report `SecretSynced` status. 6. Cilium NetworkPolicy audit log confirms deny rules active. 7. ServiceMonitor CR exists and Prometheus targets page shows sigma1 namespace targets.

## Verification Notes

- [ ] Confirm dependencies are satisfied before implementation.
- [ ] Update tests, docs, and configuration touched by this task.
- [ ] Validate the final behavior against the task objective.