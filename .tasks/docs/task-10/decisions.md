## Decision Points

- Secret rotation mechanism: CloudNative-PG built-in rotation vs external-secrets-operator — determines whether a unified rotation approach is used across all services or CNPG-specific tooling for PostgreSQL
- Secret encryption at rest strategy: rely on cluster-level etcd encryption (if available) vs implement SealedSecrets vs external-secrets-operator — depends on cluster capabilities and team operational preferences
- Container image scanning integration: Trivy vs Snyk vs Grype — choose based on existing CI tooling and license requirements

## Coordination Notes

- Agent owner: bolt
- Primary stack: Kubernetes/Helm