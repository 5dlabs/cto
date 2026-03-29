## Acceptance Criteria

- [ ] 1. Verify `databases` and `sigma1` namespaces exist.2. Confirm `sigma1-postgres` Cluster and `sigma1-valkey` Redis instances are running and accessible within the cluster.3. Validate that the `sigma1-infra-endpoints` ConfigMap exists in the `sigma1` namespace and contains correct, accessible connection URLs for PostgreSQL and Redis/Valkey.4. Test S3/R2 access by attempting to create/read a dummy object using configured credentials.

## Verification Notes

- [ ] Confirm dependencies are satisfied before implementation.
- [ ] Update tests, docs, and configuration touched by this task.
- [ ] Validate the final behavior against the task objective.