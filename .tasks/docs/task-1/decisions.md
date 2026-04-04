## Decision Points

- The ExternalSecret CRDs reference a 'backing secret store' but the specific provider (AWS Secrets Manager, HashiCorp Vault, GCP Secret Manager, Azure Key Vault, etc.) and the SecretStore/ClusterSecretStore CR that already exists in the cluster are not specified. This affects the ExternalSecret spec.provider and authentication configuration.
- The task mentions 'Redis CR or StatefulSet' — the choice between a Redis operator CR (e.g., Spotahome redis-operator, Redis Enterprise operator) and a bare StatefulSet/Deployment affects manifest structure and operator dependencies.

## Coordination Notes

- Agent owner: bolt
- Primary stack: Kubernetes/Helm