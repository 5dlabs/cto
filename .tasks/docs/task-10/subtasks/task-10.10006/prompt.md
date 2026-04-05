Implement subtask 10006: Document access controls, audit log retention, and security policies

## Objective
Create comprehensive documentation covering all RBAC policies, secret rotation procedures, audit log retention policies, and security runbooks for operational reference and compliance.

## Steps
1. Create an RBAC matrix document listing every ServiceAccount, its namespace, assigned Roles/ClusterRoles, and the permissions granted.
2. Document the secret rotation schedule: which secrets are rotated, how often, what mechanism is used, and the manual rotation procedure for emergencies.
3. Document audit log retention policies: what is logged, where it's stored, retention periods, and how to query/search logs.
4. Create a security runbook covering:
   - How to investigate a security incident using audit logs.
   - How to revoke a compromised credential immediately.
   - How to add/modify RBAC for a new service.
   - How to verify RBAC enforcement.
5. Store all documentation in the infra repo under `docs/security/`.

## Validation
Review documentation for completeness: every ServiceAccount has an RBAC entry, every rotated secret has a documented procedure, retention policies are explicitly stated with durations. Verify runbook steps can be followed by a team member unfamiliar with the system (dry-run the procedures).