## Acceptance Criteria

- [ ] 1. `kubectl get pods -n notifycore` shows notifycore-pg and notifycore-redis pods in Running/Ready state within 120s. 2. A test Job in the namespace successfully connects to PostgreSQL (`SELECT 1` returns 1) using DATABASE_URL from the ConfigMap. 3. The same Job connects to Redis (`PING` returns `PONG`) using REDIS_URL from the ConfigMap. 4. ConfigMap `notifycore-infra-endpoints` exists and contains all four keys (DATABASE_URL, REDIS_URL, PORT, RUST_LOG) with non-empty values.

## Verification Notes

- [ ] Confirm dependencies are satisfied before implementation.
- [ ] Update tests, docs, and configuration touched by this task.
- [ ] Validate the final behavior against the task objective.