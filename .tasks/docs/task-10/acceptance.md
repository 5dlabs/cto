## Acceptance Criteria

- [ ] 1. CloudNative-PG: `status.readyInstances` == 3; simulate primary pod deletion and verify automatic failover within 30 seconds (new primary elected). 2. Redis: `redis-cli -h <sentinel> sentinel master sigma-1-redis` returns a valid master with 2 replicas. 3. Ingress: `curl -k https://<ingress-host>/api/pipeline/status` returns 200 with valid TLS certificate (verify with `openssl s_client`). 4. Network policies: attempt a connection from an unlisted pod to the PM server; assert connection is refused/timed out. Verify PM server can still reach Postgres, Redis, and bridge services. 5. RBAC: `kubectl auth can-i --as=system:serviceaccount:sigma-1-dev:sigma-1-pm-sa create pods -n sigma-1-dev` returns `no`. 6. Secret rotation: modify a secret in the backing store; assert the Kubernetes Secret is updated within `refreshInterval` (verify by comparing resourceVersion before and after). 7. `docs/production-hardening.md` exists and documents at least 6 hardening measures.

## Verification Notes

- [ ] Confirm dependencies are satisfied before implementation.
- [ ] Update tests, docs, and configuration touched by this task.
- [ ] Validate the final behavior against the task objective.