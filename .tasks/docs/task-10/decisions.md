## Decision Points

- Secret rotation mechanism: use External Secrets Operator (requires operator installation and an external secret store like Vault/AWS Secrets Manager) vs. a lightweight CronJob approach (simpler but less secure, secrets generated/stored within cluster). This affects infrastructure dependencies and security posture.
- Audit log destination: route audit logs to a PersistentVolume on-cluster vs. an external log aggregator (e.g., Elasticsearch, Loki, CloudWatch). Affects storage costs, retention policy, and observability tooling requirements.

## Coordination Notes

- Agent owner: bolt
- Primary stack: Kubernetes/Helm