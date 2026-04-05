## Production Hardening: RBAC, Secret Rotation, Audit Logging (Bolt - Kubernetes/Helm)

### Objective
Implement RBAC, automate secret rotation, and enable audit logging for all critical infrastructure and services.

### Ownership
- Agent: Bolt
- Stack: Kubernetes/Helm
- Priority: high
- Status: pending
- Dependencies: 9

### Implementation Details
{"steps": ["Define Kubernetes RBAC roles and bindings for all service accounts.", "Integrate external secret management (e.g., Kubernetes External Secrets) for API keys and credentials.", "Automate secret rotation for all third-party integrations (Stripe, LinkedIn, etc.).", "Enable audit logging for Kubernetes API and all managed services.", "Document RBAC and secret rotation policies."]}

### Subtasks
- [ ] Define Kubernetes RBAC Roles and RoleBindings for all service accounts: Create least-privilege RBAC Role and RoleBinding (or ClusterRole/ClusterRoleBinding) manifests for every service account in the application namespace, ensuring each service can only access the resources it needs.
- [ ] Deploy and configure External Secrets Operator for centralized secret management: Install the Kubernetes External Secrets Operator (ESO) and configure SecretStore/ClusterSecretStore resources to sync secrets from the chosen external secrets backend into Kubernetes Secrets.
- [ ] Implement automated secret rotation for third-party integration credentials: Configure automated rotation schedules for all third-party API keys and credentials (Stripe, LinkedIn, etc.) with zero-downtime pod update mechanisms.
- [ ] Enable Kubernetes API audit logging: Configure the Kubernetes API server audit policy and audit log backend to capture all security-relevant API operations.
- [ ] Enable audit logging for managed services (PostgreSQL, Redis): Configure application-level audit logging for PostgreSQL and Redis to capture data access and administrative operations.
- [ ] Document RBAC policies, secret rotation procedures, and audit logging configuration: Create comprehensive documentation covering all RBAC roles and their justifications, secret rotation schedules and procedures, and audit logging configuration for operational reference.
- [ ] Penetration test RBAC and secret access controls: Perform security validation by attempting unauthorized access patterns against RBAC policies, secrets, and audit logging to verify controls are effective.