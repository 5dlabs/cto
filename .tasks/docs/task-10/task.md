## Production Hardening: RBAC, Secret Rotation, Audit Logging (Bolt - Kubernetes/Helm)

### Objective
Implement RBAC, automate secret rotation, and enable audit logging for compliance and security in the production cluster.

### Ownership
- Agent: bolt
- Stack: Kubernetes/Helm
- Priority: high
- Status: pending
- Dependencies: 9

### Implementation Details
{"steps": ["Define Kubernetes RBAC roles and bindings for all service accounts", "Integrate automated secret rotation for all sensitive credentials (PostgreSQL, Redis, API keys)", "Enable audit logging for Kubernetes API and critical services", "Configure log shipping to Grafana/Loki stack", "Document compliance procedures and test recovery from rotated secrets"]}

### Subtasks
- [ ] Define Kubernetes RBAC Roles and RoleBindings for all service accounts: Create least-privilege RBAC Roles and RoleBindings for every service account in the production namespace, ensuring each service can only access the Kubernetes API resources it needs.
- [ ] Implement automated secret rotation for PostgreSQL credentials: Configure automated rotation of PostgreSQL user credentials using CloudNative-PG's built-in secret management, ensuring application pods pick up new credentials without downtime.
- [ ] Implement automated secret rotation for Redis credentials and external API keys: Configure automated rotation of Redis passwords and external API keys (OpenAI, etc.) with zero-downtime rollover for consuming services.
- [ ] Enable Kubernetes API server audit logging: Configure the Kubernetes API server audit policy to log all administrative and sensitive actions, and ensure audit logs are written to a persistent location for collection.
- [ ] Configure log shipping from audit logs to Grafana/Loki stack: Set up log collection agents to ship Kubernetes audit logs and critical service logs to the existing Loki instance, and create Grafana dashboards for audit visibility.
- [ ] Document compliance procedures and create secret rotation runbook: Create comprehensive documentation covering RBAC policies, secret rotation procedures, audit log retention, and incident response procedures for compliance purposes.