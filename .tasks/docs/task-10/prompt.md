Implement task 10: Production Hardening: RBAC, Secret Rotation, Audit Logging (Bolt - Kubernetes/Helm)

## Goal
Implement RBAC, automate secret rotation, and enable audit logging for all services to ensure compliance and security in production.

## Task Context
- Agent owner: Bolt
- Stack: Kubernetes/Helm
- Priority: high
- Dependencies: 9

## Implementation Plan
{"steps": ["Define Kubernetes RBAC roles and bindings for each service account.", "Automate secret rotation for all API keys and database credentials using Kubernetes operators or external secret managers.", "Enable and configure audit logging for all API and database access.", "Document compliance procedures and test recovery from credential compromise."]}

## Acceptance Criteria
RBAC policies prevent unauthorized access; secrets are rotated and services reload without downtime; audit logs capture all access events; compliance documentation is complete.

## Subtasks
- Define and apply Kubernetes RBAC Roles and RoleBindings for all service accounts: Create least-privilege RBAC Roles (or ClusterRoles where necessary) and bind them to dedicated ServiceAccounts for each service, ensuring no service has more permissions than it needs.
- Implement automated secret rotation using External Secrets Operator: Deploy the External Secrets Operator (or chosen secret management solution) and configure automated rotation for all API keys, database credentials, and other sensitive values with zero-downtime reload.
- Enable and configure Kubernetes API server audit logging: Configure the Kubernetes API server audit policy to log all security-relevant events (authentication, authorization, secret access, RBAC changes) and ship audit logs to the centralized logging system.
- Configure application-level audit logging for API and database access: Enable audit logging at the application level so that all API requests and database queries from services are logged with user identity, action, resource, and timestamp, and shipped to the centralized logging system.
- Document compliance procedures and test credential compromise recovery: Write comprehensive compliance and incident response documentation covering RBAC policies, secret rotation procedures, audit log retention, and step-by-step credential compromise recovery playbook. Validate the playbook with a live drill.

## Deliverables
- Update the relevant code, configuration, and tests.
- Keep artifacts aligned with the acceptance criteria.
- Document blockers or assumptions in your final summary.