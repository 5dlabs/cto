## Decision Points

- MinIO deployment strategy: deploy a dedicated MinIO tenant per namespace via the MinIO Operator, OR create isolated buckets with independent IAM credentials on the existing MinIO cluster? This affects resource consumption, operational complexity, and isolation guarantees.
- Secrets management pattern: use SealedSecrets, external-secrets-operator, or the cluster's existing secret management solution? Must be consistent with cluster conventions.
- ArgoCD staging promotion gating: what mechanism (annotation hook, sync wave, or external webhook) integrates E2E test results with ArgoCD manual sync approval for staging?

## Coordination Notes

- Agent owner: bolt
- Primary stack: Kubernetes/Helm