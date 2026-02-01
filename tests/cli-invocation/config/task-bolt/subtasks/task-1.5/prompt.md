# Configure RBAC Policies

<task>
<agent>rbac-agent</agent>
<objective>Configure RBAC roles and role bindings for infrastructure components</objective>

<context>
Infrastructure components (databases, messaging) are deployed. Your job is to configure RBAC policies to ensure proper access control for service accounts.
</context>

<requirements>
- Create RBAC roles for each infrastructure component
- Create role bindings for service accounts
- Configure service account tokens with minimal permissions
- Enable audit logging for RBAC changes where appropriate
</requirements>

<deliverables>
- `rbac-roles.yaml` - Role definitions
- `rbac-bindings.yaml` - RoleBinding configurations
- Applied to cluster
</deliverables>

<acceptance_criteria>
- [ ] RBAC roles are applied
- [ ] Role bindings connect service accounts to roles
- [ ] Service accounts have minimal required permissions
- [ ] No cluster-admin bindings for application workloads
</acceptance_criteria>
</task>
