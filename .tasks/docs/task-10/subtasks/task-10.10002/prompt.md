Implement subtask 10002: Implement automated secret rotation for API keys and database credentials

## Objective
Deploy and configure an automated secret rotation mechanism for all sensitive credentials (database passwords, API keys, tokens) with zero-downtime updates to consuming pods.

## Steps
1. Based on decision point dp-14, choose either External Secrets Operator (ESO) with a backing secret store (e.g., HashiCorp Vault, AWS Secrets Manager) or a CronJob-based rotation approach.
2. If ESO: deploy the External Secrets Operator Helm chart, create SecretStore/ClusterSecretStore CR pointing to the backing store, create ExternalSecret CRs for each managed secret with `refreshInterval` for automatic rotation.
3. If manual rotation automation: create a CronJob per secret that generates new credentials, updates the Kubernetes Secret, and triggers a rolling restart of consuming Deployments via `kubectl rollout restart`.
4. For database credentials specifically: implement a dual-credential rotation pattern — create new credentials in the DB, update the Secret, wait for all pods to pick up the new credential, then revoke the old one.
5. Ensure all Deployments reference secrets via `envFrom` or volume mounts with proper `annotations` to trigger rolling updates on secret change (e.g., Reloader or stakater/Reloader operator).
6. Store rotation manifests under `infra/secret-rotation/`.

## Validation
Trigger a manual rotation cycle and verify: (a) new secret values are generated and stored in the Kubernetes Secret, (b) all consuming pods receive the updated secret without restart failures, (c) application health checks pass throughout the rotation, (d) old credentials are invalidated after rotation completes. Confirm zero-downtime by monitoring pod readiness during rotation.