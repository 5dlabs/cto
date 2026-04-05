## Production Hardening: RBAC, Secret Rotation, Audit Logging (Bolt - Kubernetes/Helm)

### Objective
Implement RBAC, automate secret rotation, and enable audit logging for all services to ensure security and compliance.

### Ownership
- Agent: bolt
- Stack: Kubernetes/Helm
- Priority: high
- Status: pending
- Dependencies: 9

### Implementation Details
{"steps":["Define Kubernetes RBAC roles and role bindings for all namespaces and services.","Automate secret rotation for API keys, database credentials, and service tokens using Kubernetes operators or external secret managers.","Enable audit logging for all service access and API calls.","Integrate with existing observability stack (Grafana, Loki, Prometheus) for monitoring and alerting.","Document RBAC and audit policies for compliance."]}

### Subtasks
- [ ] Define Kubernetes RBAC roles and role bindings for all namespaces and services: Create fine-grained RBAC Role, ClusterRole, RoleBinding, and ClusterRoleBinding resources for all namespaces, ensuring each service's ServiceAccount has least-privilege access to only the Kubernetes resources it needs.
- [ ] Automate secret rotation for database credentials: Implement automated rotation of PostgreSQL and Redis credentials using the External Secrets Operator or equivalent, ensuring services pick up new credentials without downtime.
- [ ] Automate secret rotation for API keys and service tokens: Implement automated rotation for application-level API keys, inter-service tokens, and third-party API credentials, ensuring all consuming services are updated seamlessly.
- [ ] Enable Kubernetes API audit logging: Configure the Kubernetes API server audit policy to log all authentication, authorization, and resource mutation events, and ship audit logs to the existing log aggregation stack (Loki).
- [ ] Enable application-level audit logging for all services: Configure all backend services (Equipment Catalog, RMS, Finance, Vetting, Social, Morgan) to emit structured audit logs for API access events, and ship these logs to Loki for centralized analysis.
- [ ] Create Grafana dashboards and Prometheus alerting rules for security monitoring: Build Grafana dashboards for RBAC violations, secret rotation status, and audit log analysis. Create Prometheus alerting rules for security-relevant events such as failed auth attempts, secret expiry warnings, and unauthorized access patterns.
- [ ] Document RBAC policies, audit procedures, and compliance verification: Create comprehensive documentation covering all RBAC policies, secret rotation procedures, audit logging architecture, and compliance verification steps for operational handoff and audit readiness.