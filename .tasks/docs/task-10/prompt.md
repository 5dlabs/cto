Implement task 10: Production Hardening: RBAC, Secret Rotation, and Audit Logging (Bolt - Kubernetes/Helm)

## Goal
Stretch/deferred task: Implement RBAC restrictions for the sigma-1 namespace, configure automated secret rotation via the external-secrets operator, and enable audit logging for pipeline operations. This task is beyond the PRD's stated acceptance criteria and should only be started after Task 9 is complete.

## Task Context
- Agent owner: bolt
- Stack: Kubernetes/Helm
- Priority: medium
- Dependencies: 9

## Implementation Plan
Step-by-step implementation:

1. RBAC configuration:
   - Create a ServiceAccount `sigma-1-pipeline-sa` for cto-pm pods
   - Create a Role `sigma-1-pipeline-role` with least-privilege permissions:
     - Read access: ConfigMaps, Secrets (only sigma-1 namespace)
     - No write access to Secrets (managed by external-secrets operator only)
     - No access to other namespaces
   - Create RoleBinding binding the SA to the Role
   - Update cto-pm Deployment to use `sigma-1-pipeline-sa`
   - Create a ClusterRole for cross-namespace read access to bridge services (if needed)

2. Secret rotation:
   - Configure ExternalSecret resources with `refreshInterval: 1h` (or organization standard)
   - Ensure cto-pm handles secret rotation gracefully:
     - Linear token rotation: test that new token is picked up on next API call (env var reload or volume watch)
     - NOUS_API_KEY rotation: same pattern
     - GitHub token rotation: same pattern
   - Add a rotation validation CronJob that runs daily:
     - Checks ExternalSecret sync status
     - Verifies rotated secrets are non-empty
     - Alerts via Discord bridge if rotation fails

3. Audit logging:
   - Enable Kubernetes audit policy for the sigma-1 namespace:
     - Log all create/delete/update operations on Issues (Linear API calls logged at app layer)
     - Log all Secret access events
     - Log all RBAC-related events (authentication, authorization)
   - Configure cto-pm application-level audit logging:
     - Log every delegation resolution with timestamp, agent hint, resolved user ID
     - Log every Linear issue creation with issue ID, assignee ID
     - Log every notification sent with bridge, payload hash, response status
     - Log pipeline stage transitions with timestamps
   - Ship logs to cluster logging infrastructure (if available: EFK, Loki, etc.)

4. Security scanning:
   - Add a CronJob that runs `trivy` or equivalent scanner against cto-pm container image weekly
   - Alert on HIGH/CRITICAL vulnerabilities via Discord bridge

5. Documentation:
   - Create `SECURITY.md` in the sigma-1 repo documenting:
     - RBAC roles and their permissions
     - Secret rotation schedule and procedure
     - Audit log locations and retention policy
     - Incident response contacts

## Acceptance Criteria
1. RBAC: `kubectl auth can-i get secrets -n sigma-1 --as=system:serviceaccount:sigma-1:sigma-1-pipeline-sa` returns 'yes'. 2. RBAC: `kubectl auth can-i create secrets -n sigma-1 --as=system:serviceaccount:sigma-1:sigma-1-pipeline-sa` returns 'no' (write denied). 3. RBAC: `kubectl auth can-i get secrets -n default --as=system:serviceaccount:sigma-1:sigma-1-pipeline-sa` returns 'no' (cross-namespace denied). 4. Secret rotation: ExternalSecret resources show `refreshInterval` configured and `lastSyncedTime` within the last refresh interval. 5. Audit logging: trigger a pipeline run and verify application logs contain entries for delegation resolution, issue creation, and notification events — each with timestamps. 6. Rotation CronJob: `kubectl get cronjob -n sigma-1` shows rotation-validation job with schedule and last successful run. 7. Security scan CronJob exists and last run produced a report (even if no vulnerabilities found). 8. `SECURITY.md` exists in the sigma-1 repo with sections for RBAC, rotation, audit, and incident response.

## Subtasks
- Create RBAC ServiceAccount and Role with least-privilege permissions: Create the ServiceAccount `sigma-1-pipeline-sa`, a Role `sigma-1-pipeline-role` with read-only access to ConfigMaps and Secrets in the sigma-1 namespace (no write access to Secrets), and a RoleBinding linking them together. All manifests go in the sigma-1 Helm chart or kustomize overlay.
- Update cto-pm Deployment to use sigma-1-pipeline-sa ServiceAccount: Patch the cto-pm Deployment spec to reference the new ServiceAccount `sigma-1-pipeline-sa`, replacing the default service account. Also set `automountServiceAccountToken: true` only if needed, or `false` if the pod does not require Kubernetes API access.
- Configure ExternalSecret resources with refreshInterval for automated secret rotation: Update all ExternalSecret CRs in the sigma-1 namespace to include a `refreshInterval` (e.g., 1h) so that secrets are periodically re-synced from the external secret store, enabling automated rotation.
- Create rotation validation CronJob with Discord alerting: Create a Kubernetes CronJob that runs daily to validate ExternalSecret sync status, verify rotated secrets are non-empty, and alert via the Discord bridge if any rotation has failed.
- Configure Kubernetes audit policy for sigma-1 namespace: Create or update the Kubernetes API server audit policy to capture security-relevant events in the sigma-1 namespace: Secret access, RBAC events, and mutating operations on critical resources.
- Implement application-level audit logging for cto-pm pipeline operations: Add structured audit log entries to the cto-pm application for all key pipeline events: delegation resolution, Linear issue creation, notification dispatch, and pipeline stage transitions. Each log entry must include a timestamp, event type, and relevant identifiers.
- Configure log shipping to cluster logging infrastructure: Ensure audit logs (both Kubernetes-level and application-level) from the sigma-1 namespace are shipped to the cluster's central logging infrastructure (EFK, Loki, or equivalent) with appropriate labels and retention.
- Create security scanning CronJob with Trivy: Add a Kubernetes CronJob that runs Trivy (or equivalent vulnerability scanner) weekly against the cto-pm container image and alerts via the Discord bridge on HIGH/CRITICAL findings.
- Create SECURITY.md documentation for sigma-1: Write comprehensive SECURITY.md documentation covering RBAC roles and permissions, secret rotation schedule and procedures, audit log locations and retention policy, and incident response contacts.

## Deliverables
- Update the relevant code, configuration, and tests.
- Keep artifacts aligned with the acceptance criteria.
- Document blockers or assumptions in your final summary.