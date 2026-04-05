Implement task 10: Production Hardening: RBAC, Secret Rotation, Audit Logging (Bolt - Kubernetes/Helm)

## Goal
Implement RBAC, automate secret rotation, and enable audit logging for all critical infrastructure and services.

## Task Context
- Agent owner: Bolt
- Stack: Kubernetes/Helm
- Priority: high
- Dependencies: 9

## Implementation Plan
{"steps": ["Define Kubernetes RBAC roles and bindings for all service accounts.", "Integrate external secret management (e.g., Kubernetes External Secrets) for API keys and credentials.", "Automate secret rotation for all third-party integrations (Stripe, LinkedIn, etc.).", "Enable audit logging for Kubernetes API and all managed services.", "Document RBAC and secret rotation policies."]}

## Acceptance Criteria
RBAC policies prevent unauthorized access; secrets rotate on schedule and are updated in pods; audit logs capture all access and changes; attempt unauthorized actions and verify they are blocked and logged.

## Subtasks
- Define Kubernetes RBAC Roles and RoleBindings for all service accounts: Create least-privilege RBAC Role and RoleBinding (or ClusterRole/ClusterRoleBinding) manifests for every service account in the application namespace, ensuring each service can only access the resources it needs.
- Deploy and configure External Secrets Operator for centralized secret management: Install the Kubernetes External Secrets Operator (ESO) and configure SecretStore/ClusterSecretStore resources to sync secrets from the chosen external secrets backend into Kubernetes Secrets.
- Implement automated secret rotation for third-party integration credentials: Configure automated rotation schedules for all third-party API keys and credentials (Stripe, LinkedIn, etc.) with zero-downtime pod update mechanisms.
- Enable Kubernetes API audit logging: Configure the Kubernetes API server audit policy and audit log backend to capture all security-relevant API operations.
- Enable audit logging for managed services (PostgreSQL, Redis): Configure application-level audit logging for PostgreSQL and Redis to capture data access and administrative operations.
- Document RBAC policies, secret rotation procedures, and audit logging configuration: Create comprehensive documentation covering all RBAC roles and their justifications, secret rotation schedules and procedures, and audit logging configuration for operational reference.
- Penetration test RBAC and secret access controls: Perform security validation by attempting unauthorized access patterns against RBAC policies, secrets, and audit logging to verify controls are effective.

## Deliverables
- Update the relevant code, configuration, and tests.
- Keep artifacts aligned with the acceptance criteria.
- Document blockers or assumptions in your final summary.