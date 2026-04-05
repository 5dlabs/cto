## Decision Points

- What authentication and authorization mechanism should be used for internal service-to-service and external API access?
- What is the access control model for admin endpoints (e.g., product add/update, payroll entry, vetting pipeline)?
- Which external secret management provider/backend should be used with Kubernetes External Secrets Operator?
- What audit log retention period and storage backend should be used for Kubernetes API audit logs?

## Coordination Notes

- Agent owner: Bolt
- Primary stack: Kubernetes/Helm