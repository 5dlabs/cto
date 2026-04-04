Implement subtask 10009: Create SECURITY.md documentation for sigma-1

## Objective
Write comprehensive SECURITY.md documentation covering RBAC roles and permissions, secret rotation schedule and procedures, audit log locations and retention policy, and incident response contacts.

## Steps
Step-by-step:
1. Create `SECURITY.md` at the root of the sigma-1 repository.
2. Sections to include:
   a. **RBAC Overview**: List all ServiceAccounts, Roles, ClusterRoles, and RoleBindings. For each Role, list the exact permissions (resources, verbs). Explain the least-privilege rationale.
   b. **Secret Rotation**: Document which secrets are managed by ExternalSecrets, the refresh interval, how rotation is validated (CronJob), and the manual rotation procedure for emergencies.
   c. **Audit Logging**: Document both Kubernetes-level and application-level audit events. List event types, where logs are shipped, how to query them, and the retention policy.
   d. **Security Scanning**: Document the Trivy CronJob schedule, how to interpret results, and the alerting flow.
   e. **Incident Response**: List contacts (team, escalation path), link to runbooks, and document the process for responding to a compromised secret or critical vulnerability.
3. Use clear markdown formatting with code blocks for kubectl commands.
4. Review for accuracy against the actual manifests created in prior subtasks.

## Validation
Verify `SECURITY.md` exists at the repo root. Confirm it contains all five sections: RBAC Overview, Secret Rotation, Audit Logging, Security Scanning, and Incident Response. Each section must contain specific details (not placeholders) — e.g., RBAC section lists actual Role names and permissions. All kubectl commands in the document execute without syntax errors.