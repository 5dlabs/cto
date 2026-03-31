Implement subtask 8002: Create rollback procedures document

## Objective
Write rollback-procedures.md covering immediate feature flag rollback, database rollback, MinIO cleanup, trigger conditions linked to Grafana alerts, and post-rollback verification checklist.

## Steps
1. Create `docs/hermes/rollback-procedures.md` with these sections:
   - **Immediate Rollback (Feature Flag)**:
     - Exact commands: `kubectl edit configmap hermes-config -n <namespace>` to set `HERMES_ENABLED=false`, or patch command
     - ArgoCD sync command to propagate: `argocd app sync hermes-backend-staging`
     - Expected behavior: Hermes routes return 404 within 60 seconds, no data loss
     - Verification: `curl -s -o /dev/null -w '%{http_code}' $BASE_URL/api/hermes/deliberations` returns 404
   - **Database Rollback**:
     - Note: tables are additive only (per D6), so rollback = drop tables
     - SQL commands: `DROP TABLE IF EXISTS hermes_artifacts CASCADE; DROP TABLE IF EXISTS deliberations CASCADE;`
     - Warning about data loss implications
     - When to use: only for full feature removal, not routine rollback
   - **MinIO Cleanup**:
     - `mc rb --force minio/hermes-artifacts` command
     - Only needed for complete rollback; bucket can safely remain for feature re-enable
   - **Rollback Trigger Conditions**:
     - Link to Grafana dashboard from Task 6
     - Specific alert thresholds: error rate > 5% for 5 minutes, artifact generation failure > 10%, API p99 latency > 5s
   - **Post-Rollback Verification Checklist**:
     - 8-10 items: API returns 404, nav item hidden, no orphaned processes, monitoring shows zero Hermes traffic, etc.

## Validation
Verify rollback-procedures.md exists and contains all 5 sections. Execute the immediate rollback procedure in staging: patch ConfigMap to set HERMES_ENABLED=false, run ArgoCD sync, then verify `/api/hermes/deliberations` returns 404 within 120 seconds. Restore HERMES_ENABLED=true afterward.