Implement task 10: Production Hardening: RBAC, Secret Rotation, Audit Logging (Bolt - Kubernetes/Helm)

## Goal
Implement production security hardening — Kubernetes RBAC policies, automated secret rotation for all data service credentials, comprehensive audit logging for critical resource access, and security scanning integration for the Hermes pipeline.

## Task Context
- Agent owner: bolt
- Stack: Kubernetes/Helm
- Priority: high
- Dependencies: 1, 2, 3, 4, 5, 6, 7, 8, 9

## Implementation Plan
Step-by-step implementation:

1. **Kubernetes RBAC:**
   - Create `ServiceAccount` for each Hermes service (backend, frontend) in production namespace
   - Create `Role` and `RoleBinding` restricting each ServiceAccount to only the resources it needs:
     - Backend SA: get/list ConfigMaps, get Secrets (hermes-* only), no cluster-level access
     - Frontend SA: get/list ConfigMaps only
   - Create `ClusterRole` for Hermes operators (human admins): full access to hermes-production namespace
   - Verify no default ServiceAccount token automount (`automountServiceAccountToken: false` on pods)

2. **Application-level RBAC claims (coordination with Task 2):**
   - Finalize RBAC claim taxonomy:
     - `hermes:read` — view deliberations and artifacts
     - `hermes:trigger` — create new deliberations
     - `hermes:admin` — trigger migrations, access admin endpoints
     - `hermes:delete` — delete deliberations and artifacts (optional, for future use)
   - Ensure claims are stored in the session model and checked by the Hermes middleware (Task 2)
   - Create a database migration to add claims to existing admin users

3. **Automated secret rotation:**
   - Implement secret rotation for PostgreSQL credentials:
     - Use CloudNative-PG's built-in rotation mechanism or external-secrets-operator
     - Rotation frequency: every 90 days for production
     - Zero-downtime rotation: new credentials provisioned, services restarted via rolling update, old credentials revoked
   - Implement secret rotation for MinIO credentials:
     - Create rotation CronJob or use external-secrets-operator
     - Test that artifact read/write continues working after rotation
   - Implement secret rotation for Redis credentials:
     - Similar pattern to PostgreSQL
   - Store rotation schedule and last rotation timestamp as annotations on Secret objects

4. **Audit logging:**
   - Enable Kubernetes audit logging for the `hermes-production` namespace (if not already enabled at cluster level)
   - Application-level audit logging: log all critical resource access with structured fields:
     - `audit_action`: `create_deliberation`, `read_artifact`, `trigger_migration`, `access_admin_endpoint`
     - `audit_actor`: user ID from session
     - `audit_resource`: resource type and ID
     - `audit_result`: `success` | `denied` | `error`
     - `audit_ip`: client IP address
   - Implement audit logging middleware in `src/modules/hermes/middleware.ts` (extends Task 2's auth middleware)
   - Audit logs must be shipped to Loki with a dedicated label (`audit=true`) for separate retention policy

5. **Pod security:**
   - Apply `PodSecurityStandard: restricted` or equivalent SecurityContext on all Hermes pods:
     - `runAsNonRoot: true`
     - `readOnlyRootFilesystem: true` (with writable tmpdir mount for headless browser)
     - `allowPrivilegeEscalation: false`
     - Drop all capabilities
   - Scan container images for vulnerabilities (integrate Trivy or similar into CI)

6. **Secret encryption at rest:**
   - Verify Kubernetes secrets are encrypted at rest (etcd encryption) — document current state
   - If not encrypted, implement etcd encryption configuration or use SealedSecrets/external-secrets-operator

7. **Production readiness checklist:** Create `docs/hermes/production-readiness-checklist.md`:
   - All RBAC policies applied
   - All secrets rotated at least once
   - Audit logging verified in Loki
   - Pod security contexts applied
   - Network policies verified (from Task 9)
   - TLS verified (from Task 9)
   - E2E tests passing (from Task 7)
   - Rollback procedures documented (from Task 8)

## Acceptance Criteria
1. RBAC enforcement: The backend ServiceAccount can read `hermes-infra-endpoints` ConfigMap but CANNOT list Secrets in other namespaces — verified by `kubectl auth can-i` with the SA token.
2. Application RBAC: A user with only `hermes:read` claim receives 403 when calling `POST /api/hermes/deliberations` and 200 when calling `GET /api/hermes/deliberations`.
3. Secret rotation: After rotating PostgreSQL credentials via the rotation mechanism, the Hermes backend service continues to serve requests within 60 seconds (rolling restart completes, no 500 errors during rotation window).
4. Audit logging: Create a deliberation via the API; within 30 seconds, query Loki with `{app="hermes", audit="true"} | json | audit_action="create_deliberation"` and verify the log entry contains the correct `audit_actor` (user ID) and `audit_resource` (deliberation ID).
5. Pod security: `kubectl get pod -n hermes-production -o jsonpath='{.items[0].spec.containers[0].securityContext}'` shows `runAsNonRoot: true` and `allowPrivilegeEscalation: false` for all Hermes pods.
6. Production readiness: All items in `docs/hermes/production-readiness-checklist.md` are marked complete with evidence links (Loki queries, kubectl outputs, or CI run URLs).
7. Claim taxonomy: The `hermes:admin` claim gates access to `POST /api/hermes/admin/migrate-artifacts` — verified by 403 without claim and 202 with claim.

## Subtasks
- Create Kubernetes RBAC: ServiceAccounts, Roles, and RoleBindings for Hermes services: Create dedicated ServiceAccounts for backend and frontend services, create Roles with least-privilege access, bind them with RoleBindings, and disable default ServiceAccount token automount on all pods.
- Create ClusterRole for Hermes operator (human admin) access: Create a ClusterRole and ClusterRoleBinding for human Hermes operators granting full access to the hermes-production namespace resources.
- Finalize application-level RBAC claim taxonomy and create database migration: Define the RBAC claim taxonomy (hermes:read, hermes:trigger, hermes:admin, hermes:delete), coordinate with Task 2's session model, and create a database migration to add claims to existing admin users.
- Implement automated secret rotation for PostgreSQL credentials: Configure automated 90-day secret rotation for PostgreSQL credentials using CloudNative-PG's built-in mechanism or external-secrets-operator, with zero-downtime rolling restart of dependent services.
- Implement automated secret rotation for Redis credentials: Configure automated 90-day secret rotation for Redis credentials with zero-downtime rolling restart of dependent services.
- Implement automated secret rotation for MinIO credentials: Configure automated 90-day secret rotation for MinIO access keys with zero-downtime rolling restart and verification that artifact read/write continues working.
- Implement application-level audit logging middleware with Loki integration: Create audit logging middleware in the Hermes backend that logs all critical resource access with structured fields (audit_action, audit_actor, audit_resource, audit_result, audit_ip) and ships logs to Loki with a dedicated 'audit=true' label.
- Apply Pod Security contexts to all Hermes pods: Configure SecurityContext on all Hermes pod specs with runAsNonRoot, readOnlyRootFilesystem (with writable tmpdir for headless browser), allowPrivilegeEscalation=false, and drop all capabilities.
- Integrate container image vulnerability scanning with Trivy in CI: Add Trivy (or chosen scanner) to the CI pipeline to scan all Hermes container images for vulnerabilities before deployment, failing the build on critical/high severity findings.
- Verify and document secret encryption at rest (etcd encryption): Verify whether Kubernetes secrets are encrypted at rest in etcd, document the current state, and implement encryption if not already configured (or document the path to SealedSecrets/external-secrets-operator).
- Create production readiness checklist document: Create docs/hermes/production-readiness-checklist.md covering all security, reliability, and operational readiness items with evidence links and verification commands.

## Deliverables
- Update the relevant code, configuration, and tests.
- Keep artifacts aligned with the acceptance criteria.
- Document blockers or assumptions in your final summary.