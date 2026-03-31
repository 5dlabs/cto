Implement subtask 10010: Verify and document secret encryption at rest (etcd encryption)

## Objective
Verify whether Kubernetes secrets are encrypted at rest in etcd, document the current state, and implement encryption if not already configured (or document the path to SealedSecrets/external-secrets-operator).

## Steps
1. Check if etcd encryption is configured at the cluster level:
   - For managed Kubernetes (EKS, GKE, AKS): document the provider's encryption-at-rest status (usually enabled by default).
   - For self-managed clusters: check `kube-apiserver` flags for `--encryption-provider-config`.
2. Document findings in `docs/hermes/secret-encryption-status.md`.
3. If encryption is NOT enabled:
   - Option A: Configure etcd encryption via EncryptionConfiguration (requires API server restart — may not be feasible for managed clusters).
   - Option B: Implement SealedSecrets: install the SealedSecrets controller, convert all Hermes secrets to SealedSecret CRs that can be safely stored in Git.
   - Option C: Use external-secrets-operator with a secrets manager (Vault, AWS Secrets Manager, etc.).
4. If encryption IS enabled: document the encryption method and key rotation schedule.
5. Ensure all Hermes secrets (PostgreSQL, Redis, MinIO, TLS) are covered.

## Validation
Verify `docs/hermes/secret-encryption-status.md` exists with clear documentation. If using managed Kubernetes, verify the provider's documentation confirms encryption at rest. If SealedSecrets are implemented, verify `kubectl get sealedsecrets -n hermes-production` lists all Hermes secrets and the corresponding Secret objects are created.