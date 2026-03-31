Implement subtask 8004: Create operational runbook with LogQL queries

## Objective
Write runbook.md covering common Hermes operational issues, resolution steps, validated LogQL queries for troubleshooting, and an escalation matrix.

## Steps
1. Create `docs/hermes/runbook.md` with these sections:
   - **Common Issues and Resolutions** (table format with columns: Symptom, Cause, Resolution, Severity):
     a. Deliberation stuck in `processing` → headless browser pod OOM or crash → check pod logs (`kubectl logs -l app=hermes-worker -n <ns> --tail=100`), restart pod, check memory limits
     b. Artifact upload failures → MinIO unhealthy or credentials expired → check MinIO health (`mc admin info minio`), verify secret rotation status
     c. High latency on artifact retrieval → presigned URL TTL issues or MinIO performance → check TTL config, MinIO metrics dashboard
     d. 403 errors after deployment → RBAC claims not propagated → verify session claims, check auth service logs
     e. E2E tests failing in CI → staging environment drift → compare deployed image tags, check ConfigMap values
   - **LogQL Queries** (at least 3, validated):
     a. Hermes API error logs: `{namespace="hermes", app="hermes-backend"} |= "error" | json | level="error"` — filter for errors in the last 1h
     b. Deliberation processing duration: `{namespace="hermes", app="hermes-worker"} |= "deliberation_completed" | json | unwrap duration_ms` — histogram of processing times
     c. Artifact upload failures: `{namespace="hermes", app="hermes-backend"} |= "upload_failed" | json | line_format "{{.deliberation_id}} {{.error}}"` — identify failing deliberations
   - **Useful kubectl Commands**: pod status, log tailing, exec into pod, port-forward to MinIO
   - **Escalation Matrix**: table with severity levels (P1-P4), response time, team/person, communication channel

## Validation
Verify runbook.md exists with all specified sections. Run at least 3 LogQL queries against Loki in the staging environment and confirm they are syntactically valid (return results or empty result set without parse errors). Verify kubectl commands use correct label selectors matching the deployed Hermes resources.