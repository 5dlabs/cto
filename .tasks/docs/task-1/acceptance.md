## Acceptance Criteria

- [ ] 1. `kubectl get cluster sigma1-postgres -n sigma1-db` shows READY with 2/2 instances healthy. 2. `kubectl exec` into a sigma1 pod and verify `psql` connection via PgBouncer pooler URL succeeds and `\dn` lists all 6 schemas (catalog, rms, finance, vetting, social, audit). 3. `redis-cli -u $VALKEY_URL PING` returns PONG. 4. ConfigMap `sigma1-infra-endpoints` exists with all 5 expected keys. 5. All Kubernetes Secrets exist with non-empty data keys. 6. Cloudflare Tunnel pod is Running and tunnel status shows CONNECTED. 7. ServiceMonitor CRs are picked up by Prometheus (check Prometheus targets page). 8. PgBouncer stats show active connection pools when queried via `SHOW POOLS`.

## Verification Notes

- [ ] Confirm dependencies are satisfied before implementation.
- [ ] Update tests, docs, and configuration touched by this task.
- [ ] Validate the final behavior against the task objective.