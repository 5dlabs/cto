## Acceptance Criteria

- [ ] 1. HA PostgreSQL: `kubectl get pods -n hermes-production -l cnpg.io/cluster=hermes-pg` returns 3 Running pods. Killing the primary pod results in automatic failover to a replica within 30 seconds (verified by continuous query during failover).
- [ ] 2. TLS: `curl -v https://hermes.{domain}/api/hermes/deliberations` shows TLS 1.2+ handshake and valid certificate. `curl http://hermes.{domain}` returns 301 redirect to HTTPS.
- [ ] 3. Network policy: A test pod in `hermes-production` namespace can reach PostgreSQL on port 5432 but cannot reach the Kubernetes API server or pods in other namespaces (verified by `kubectl exec` with `curl` and `nc`).
- [ ] 4. Autoscaling: Under load (50 concurrent deliberation requests), HPA scales the backend service beyond 2 replicas within 3 minutes. After load subsides, replicas scale back to 2 within 10 minutes.
- [ ] 5. PodDisruptionBudget: `kubectl get pdb -n hermes-production` shows PDBs for all services with `ALLOWED DISRUPTIONS >= 1`.
- [ ] 6. Ingress routing: `curl https://hermes.{domain}/` returns the Next.js application HTML. `curl https://hermes.{domain}/api/hermes/deliberations` (or `hermes-api.{domain}`) returns JSON from the Elysia service.
- [ ] 7. MinIO versioning: Deleting an object from the production Hermes bucket and then listing versions shows the deleted object as a delete marker with the previous version recoverable.

## Verification Notes

- [ ] Confirm dependencies are satisfied before implementation.
- [ ] Update tests, docs, and configuration touched by this task.
- [ ] Validate the final behavior against the task objective.