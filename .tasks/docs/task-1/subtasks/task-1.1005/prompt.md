Implement subtask 1005: Provision dedicated MinIO bucket with IAM credentials and lifecycle policies

## Objective
Create a dedicated MinIO bucket for Hermes artifacts in each namespace with independent IAM credentials and configurable retention lifecycle policies. Must NOT reuse GitLab-owned buckets.

## Steps
1. Based on the chosen MinIO strategy (dedicated tenant or shared cluster with isolated bucket), create the appropriate Helm template in `charts/hermes-infra/templates/minio.yaml`.
2. For dedicated tenant approach: deploy a MinIO Tenant CR with single-replica in each namespace.
3. For shared cluster approach: use a Job or init-container that calls the MinIO admin API to create buckets `hermes-artifacts-dev` and `hermes-artifacts-staging` with dedicated IAM users.
4. Create lifecycle policy: 90-day object expiration for dev (configurable via `values-dev.yaml`), 365-day for staging (configurable via `values-staging.yaml`).
5. Store S3 endpoint, access key, secret key, and bucket name in secret `hermes-minio-credentials` in each namespace.
6. CRITICAL: Verify the generated IAM credentials do NOT have access to any bucket outside `hermes-artifacts-*`. Test by attempting to list/read a known GitLab bucket — must return Access Denied.

## Validation
MinIO bucket `hermes-artifacts-dev` exists and is writable with dedicated credentials from `hermes-minio-credentials`. A `mc ls` against any GitLab-owned bucket using the Hermes credentials returns Access Denied. Lifecycle policy is confirmed via `mc ilm ls` showing the configured retention.