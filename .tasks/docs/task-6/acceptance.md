## Acceptance Criteria

- [ ] 1. Structured log validation: All Hermes API requests generate logs queryable in Loki via `{app="hermes"} | json | module="hermes"` — each entry contains `rollout_phase`, `operation`, and `duration_ms` fields.
- [ ] 2. Rollback trigger: Simulate 25 consecutive deliberation failures; verify that a log entry with `rollback_trigger=true` and `error_code=FAILURE_RATE_EXCEEDED` appears in Loki within 1 minute.
- [ ] 3. MinIO health monitoring: Stop the MinIO service for 4 minutes; verify that logs with `minio_health="unreachable"` appear and a `rollback_trigger` log is emitted after 3 consecutive failures.
- [ ] 4. Grafana dashboards: The Rollout Health Dashboard loads in Grafana without errors and displays at least one data point when the Hermes service has been running for >5 minutes.
- [ ] 5. LogQL queryability: The query `{app="hermes"} | json | migration_step="complete"` returns results after a migration run (cross-validates with Task 5).
- [ ] 6. Error code taxonomy: Deliberation errors, artifact write errors, and migration errors each use distinct `error_code` values — verified by querying `{app="hermes"} | json | error_code!="" | line_format "{{.error_code}}"` and confirming at least 3 distinct codes in test runs.

## Verification Notes

- [ ] Confirm dependencies are satisfied before implementation.
- [ ] Update tests, docs, and configuration touched by this task.
- [ ] Validate the final behavior against the task objective.