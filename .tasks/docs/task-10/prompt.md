Implement task 10: Production Hardening: RBAC, Secret Rotation, Audit Logging (Bolt - Kubernetes/Helm)

## Goal
Implement RBAC, automate secret rotation, and enable audit logging for all critical infrastructure and services. Ensures compliance, security, and traceability.

## Task Context
- Agent owner: bolt
- Stack: Kubernetes/Helm
- Priority: high
- Dependencies: 9

## Implementation Plan
{"steps": ["Define Kubernetes RBAC roles and bindings for all service accounts.", "Automate secret rotation for API keys and database credentials.", "Enable audit logging for Kubernetes API and all managed services.", "Integrate with existing security scanning and monitoring tools.", "Document compliance and traceability procedures."]}

## Acceptance Criteria
RBAC policies prevent unauthorized access. Secrets are rotated and updated in all pods without downtime. Audit logs capture all access and configuration changes. Security scans show no critical vulnerabilities.

## Subtasks
- Define Kubernetes RBAC Roles and RoleBindings for all service accounts: Create least-privilege RBAC Roles, ClusterRoles, RoleBindings, and ClusterRoleBindings for every service account in the cluster. Each service should only have access to the namespaced resources it needs.
- Implement automated secret rotation for API keys and database credentials: Deploy and configure an automated secret rotation mechanism for all sensitive credentials (database passwords, API keys, tokens) with zero-downtime updates to consuming pods.
- Enable Kubernetes API server audit logging with policy configuration: Configure the Kubernetes API server audit logging with an appropriate audit policy that captures security-relevant events (authentication, authorization, secret access, RBAC changes) while filtering noise.
- Configure audit log aggregation and forwarding to monitoring stack: Forward Kubernetes audit logs to the existing log aggregation/monitoring stack for centralized querying, alerting, and long-term retention.
- Integrate security scanning and validate compliance posture: Run security scanning tools against the hardened cluster to validate RBAC policies, secret management, and audit logging configurations meet security baselines.

## Deliverables
- Update the relevant code, configuration, and tests.
- Keep artifacts aligned with the acceptance criteria.
- Document blockers or assumptions in your final summary.