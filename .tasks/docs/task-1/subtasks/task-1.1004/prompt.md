Implement subtask 1004: MinIO Capacity Gate Check Script

## Objective
Create a standalone script (and optional Kubernetes Job manifest) that checks the existing GitLab MinIO instance capacity, checks for bucket naming conflicts, and outputs a capacity report. This produces the data needed for the pre-execution MinIO decision point — it does NOT make the decision or provision anything.

## Steps
Step-by-step:
1. Create `scripts/minio-capacity-check.sh` — a standalone bash script using the `mc` CLI.
2. Script steps:
   a. Accept MinIO endpoint, root user, and root password as arguments or environment variables.
   b. `mc alias set gitlab $MINIO_ENDPOINT $MINIO_ROOT_USER $MINIO_ROOT_PASSWORD`
   c. `mc admin info gitlab --json` — parse and output: total capacity, used capacity, free capacity, utilization percentage, number of drives.
   d. `mc ls gitlab/` — check if `hermes-staging-artifacts` or `hermes-prod-artifacts` already exist. Report conflicts.
   e. Output a structured capacity report (JSON or formatted text) with: total_capacity, used_capacity, free_capacity, utilization_pct, bucket_conflicts (list), recommendation ("reuse" if free >= 50Gi AND utilization <= 70%, else "dedicated").
3. Create `charts/hermes-infra/templates/jobs/minio-capacity-check-job.yaml` — an optional Kubernetes Job using `minio/mc` image that runs the same logic. Use `helm.sh/hook: pre-install` annotation. Mount MinIO root credentials from a pre-existing Secret.
4. Job should write the capacity report to its logs (retrievable via `kubectl logs`).
5. Document in chart README: "Run this script/job BEFORE helm install. Feed the result into the minio.dedicated and minio.endpoint values."
6. This subtask does NOT provision a dedicated MinIO instance or create buckets — it only produces diagnostic data.

## Validation
Script executes successfully against the GitLab MinIO instance and outputs a valid capacity report containing total_capacity, used_capacity, free_capacity, utilization_pct, and bucket_conflicts fields. If run as a Kubernetes Job, `kubectl logs` of the completed Job shows the capacity report. Script returns exit code 0 on success, non-zero on connection failure.