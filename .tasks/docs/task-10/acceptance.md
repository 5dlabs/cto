## Acceptance Criteria

- [ ] 1. RBAC: `kubectl auth can-i get secrets -n sigma-1 --as=system:serviceaccount:sigma-1:sigma-1-pipeline-sa` returns 'yes'. 2. RBAC: `kubectl auth can-i create secrets -n sigma-1 --as=system:serviceaccount:sigma-1:sigma-1-pipeline-sa` returns 'no' (write denied). 3. RBAC: `kubectl auth can-i get secrets -n default --as=system:serviceaccount:sigma-1:sigma-1-pipeline-sa` returns 'no' (cross-namespace denied). 4. Secret rotation: ExternalSecret resources show `refreshInterval` configured and `lastSyncedTime` within the last refresh interval. 5. Audit logging: trigger a pipeline run and verify application logs contain entries for delegation resolution, issue creation, and notification events — each with timestamps. 6. Rotation CronJob: `kubectl get cronjob -n sigma-1` shows rotation-validation job with schedule and last successful run. 7. Security scan CronJob exists and last run produced a report (even if no vulnerabilities found). 8. `SECURITY.md` exists in the sigma-1 repo with sections for RBAC, rotation, audit, and incident response.

## Verification Notes

- [ ] Confirm dependencies are satisfied before implementation.
- [ ] Update tests, docs, and configuration touched by this task.
- [ ] Validate the final behavior against the task objective.