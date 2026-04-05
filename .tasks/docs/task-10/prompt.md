Implement task 10: Production Hardening: RBAC, Secret Rotation, Audit Logging (Bolt - Kubernetes/Helm)

## Goal
Enforce RBAC, automate secret rotation, and enable audit logging for all infrastructure and services to meet security and compliance requirements.

## Task Context
- Agent owner: Bolt
- Stack: Kubernetes/Helm
- Priority: high
- Dependencies: 9

## Implementation Plan
{"steps": ["Define and apply Kubernetes RBAC policies for all namespaces and service accounts.", "Automate secret rotation for all external service credentials (Stripe, LinkedIn, etc.) using Kubernetes operators or external secret managers.", "Enable and configure audit logging for Kubernetes API and all managed services.", "Integrate audit logs with centralized logging (Loki/Grafana).", "Test RBAC enforcement and secret rotation workflows.", "Document security and compliance posture."]}

## Acceptance Criteria
RBAC policies prevent unauthorized access; secret rotation completes without service downtime; audit logs are generated and visible in centralized logging; compliance checklist is met for all services.

## Subtasks
- Define and apply Kubernetes RBAC policies for all namespaces and service accounts: Create Role, ClusterRole, RoleBinding, and ClusterRoleBinding resources to enforce least-privilege access for all service accounts, operators, and human administrators across all namespaces.
- Automate secret rotation for all external service credentials: Set up automated secret rotation for external API credentials (Stripe, LinkedIn, Cloudflare, etc.) using a Kubernetes secret management operator, ensuring zero-downtime rotation.
- Enable and configure Kubernetes API server audit logging: Configure the Kubernetes API server audit policy to log security-relevant events (authentication, authorization, secret access, RBAC changes) and ship audit logs to persistent storage.
- Integrate audit logs and service logs with centralized Loki/Grafana logging: Ship Kubernetes API audit logs and managed service logs (PostgreSQL, Redis, operators) into the Loki centralized logging stack and create Grafana dashboards for security monitoring.
- Test RBAC enforcement, secret rotation workflows, and document security posture: Perform comprehensive validation of RBAC policies, secret rotation zero-downtime guarantees, audit log completeness, and produce a security/compliance documentation package.

## Deliverables
- Update the relevant code, configuration, and tests.
- Keep artifacts aligned with the acceptance criteria.
- Document blockers or assumptions in your final summary.