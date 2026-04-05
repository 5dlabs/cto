## Production Hardening: RBAC, Secret Rotation, Audit Logging (Bolt - Kubernetes/Helm)

### Objective
Implement RBAC, automate secret rotation, and enable audit logging for all critical infrastructure and services. Ensures compliance, security, and traceability.

### Ownership
- Agent: bolt
- Stack: Kubernetes/Helm
- Priority: high
- Status: pending
- Dependencies: 9

### Implementation Details
{"steps": ["Define Kubernetes RBAC roles and bindings for all service accounts.", "Integrate automated secret rotation for database, API keys, and service credentials.", "Enable audit logging for Kubernetes API and all managed services.", "Document access controls and audit log retention policies.", "Test RBAC enforcement and secret rotation workflows."]}

### Subtasks
- [ ] Define Kubernetes RBAC roles and bindings for all service accounts: Create least-privilege Role, ClusterRole, RoleBinding, and ClusterRoleBinding resources for every service account across all namespaces. Ensure no service account has more permissions than required for its function.
- [ ] Implement automated secret rotation for database credentials: Set up automated rotation for PostgreSQL database credentials (application user passwords, replication credentials) so that secrets are periodically rotated without causing application downtime.
- [ ] Implement automated secret rotation for API keys and service credentials: Set up automated rotation for external API keys (e.g., OpenAI, Cloudflare) and inter-service credentials, ensuring zero-downtime rotation with proper coordination.
- [ ] Enable Kubernetes API server audit logging with retention policy: Configure the Kubernetes API server audit policy to capture security-relevant events (authentication, authorization, secret access, RBAC changes) and ship logs to the chosen sink with defined retention.
- [ ] Enable audit logging for managed services (PostgreSQL, Redis): Configure audit logging within PostgreSQL and Redis to capture data access, administrative operations, and authentication events for compliance and forensics.
- [ ] Document access controls, audit log retention, and security policies: Create comprehensive documentation covering all RBAC policies, secret rotation procedures, audit log retention policies, and security runbooks for operational reference and compliance.
- [ ] Test RBAC enforcement and secret rotation workflows end-to-end: Execute comprehensive security tests validating that RBAC policies block unauthorized access, secret rotation works without downtime, and audit logs capture all security-relevant events during these operations.