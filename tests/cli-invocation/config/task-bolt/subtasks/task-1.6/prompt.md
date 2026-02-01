# Configure Secrets Management

<task>
<agent>secrets-agent</agent>
<objective>Configure secrets management for database credentials and sensitive data</objective>

<context>
Infrastructure components (databases, messaging) are deployed. Your job is to configure secrets for database credentials and other sensitive configuration data.
</context>

<requirements>
- Create Kubernetes secrets for database credentials (PostgreSQL, MongoDB)
- Configure secrets for Kafka authentication if needed
- Use sealed-secrets or external-secrets operator if available
- Ensure secrets are not stored in plaintext in git
</requirements>

<deliverables>
- `secrets-databases.yaml` - Database credential secrets (sealed/encrypted)
- `secrets-messaging.yaml` - Messaging secrets if needed
- Applied to cluster
</deliverables>

<acceptance_criteria>
- [ ] Database secrets are created and encrypted
- [ ] Applications can mount secrets as volumes or env vars
- [ ] No plaintext secrets in manifests
- [ ] Secret rotation policy documented
</acceptance_criteria>
</task>
