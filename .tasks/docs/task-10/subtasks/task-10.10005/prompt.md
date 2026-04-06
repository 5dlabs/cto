Implement subtask 10005: Document compliance procedures and test credential compromise recovery

## Objective
Write comprehensive compliance and incident response documentation covering RBAC policies, secret rotation procedures, audit log retention, and step-by-step credential compromise recovery playbook. Validate the playbook with a live drill.

## Steps
1) Document all RBAC policies: list every ServiceAccount, its Role/ClusterRole, and justification for each permission. 2) Document the secret rotation architecture: which secrets are managed, their rotation intervals, how services reload, and how to manually trigger an emergency rotation. 3) Document audit log retention policy: how long logs are kept, where they are stored, who can access them. 4) Write a credential compromise incident response playbook: step-by-step instructions for rotating a compromised database credential (update in external backend -> verify ExternalSecret syncs -> verify services reconnect), revoking a compromised API key, and revoking a compromised service account token (delete the ServiceAccount and recreate). 5) Execute a live drill: simulate a database credential compromise by rotating the credential and verifying all services recover automatically within the expected timeframe. Document the drill results. 6) Store all documentation in `docs/security/` in the repository.

## Validation
Compliance documentation covers all services and secrets with no gaps (review checklist). Credential compromise drill is executed successfully: database credential is rotated, services reconnect within 5 minutes with zero downtime, and the drill is documented with timestamps. A second reviewer confirms the documentation is accurate and actionable by following the playbook steps independently.