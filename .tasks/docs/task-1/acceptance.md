## Acceptance Criteria

- [ ] 1. `kubectl get clusters.postgresql.cnpg.io sigma-1-pg -n sigma-1-dev -o jsonpath='{.status.phase}'` returns `Cluster in healthy state`. 2. `kubectl get secret` lists synced secrets for all five ExternalSecret CRDs with non-empty data keys. 3. ConfigMap `sigma-1-infra-endpoints` contains exactly 4 keys with non-empty values. 4. Smoke Pod exits 0 after successfully connecting to Postgres (SELECT 1), pinging Redis (PING → PONG), and resolving both bridge service DNS names. 5. Cilium NetworkPolicy count in namespace equals expected policy count (verify with `kubectl get cnp -n sigma-1-dev`).

## Verification Notes

- [ ] Confirm dependencies are satisfied before implementation.
- [ ] Update tests, docs, and configuration touched by this task.
- [ ] Validate the final behavior against the task objective.