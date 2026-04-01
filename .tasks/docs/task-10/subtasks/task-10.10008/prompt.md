Implement subtask 10008: Create security documentation in docs/security.md

## Objective
Document all RBAC roles, ServiceAccount assignments, secret rotation schedules, NetworkPolicy rules, and audit logging configuration for operational reference and compliance.

## Steps
1. Create `docs/security.md` with the following sections:
   a. **Overview**: Summary of the production hardening measures applied to sigma1-prod.
   b. **ServiceAccounts**: Table listing each SA, its purpose, and which Deployment uses it.
   c. **RBAC Roles**: Table for each Role with columns: Role Name, Resources, Verbs, ResourceNames, Bound ServiceAccount.
   d. **Secret Rotation**: Table with columns: Secret Name, Rotation Frequency, CronJob Name, Affected Deployments, Last Rotated (placeholder).
   e. **Network Policies**: Description of each NetworkPolicy with a diagram or table showing allowed ingress/egress flows.
   f. **Audit Logging**: Description of audit policy rules, log storage location, retention policy, and how to query audit logs.
   g. **Incident Response**: Brief runbook for common scenarios (e.g., compromised secret, unauthorized access attempt detected in audit logs).
2. Commit the file to the repository root under `docs/`.

## Validation
Verify `docs/security.md` exists in the repo. Confirm it contains all required sections: ServiceAccounts table, RBAC Roles table, Secret Rotation schedule table, Network Policies description, Audit Logging configuration, and Incident Response runbook. Ensure no placeholder values remain (except 'Last Rotated' date).