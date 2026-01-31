# Configure Security Policies

<task>
<agent>security-agent</agent>
<objective>Configure security policies for the infrastructure components</objective>

<context>
Infrastructure components (databases, messaging) are deployed. Your job is to configure security policies including RBAC, secrets management, and pod security.
</context>

<requirements>
- Create RBAC roles and role bindings for service accounts
- Configure secrets for database credentials
- Set up pod security policies/standards
- Configure service account tokens
- Enable audit logging where appropriate
</requirements>

<deliverables>
- `rbac.yaml` - RBAC configuration
- `secrets.yaml` - Secrets management (sealed or external)
- `pod-security.yaml` - Pod security policies
</deliverables>

<acceptance_criteria>
- [ ] RBAC roles are applied
- [ ] Secrets are properly configured
- [ ] Pod security policies are enforced
- [ ] Service accounts have minimal required permissions
</acceptance_criteria>
</task>
