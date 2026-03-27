Implement task 1: Core Infrastructure Setup (Bolt - Kubernetes/Helm)

## Goal
Provision the foundational Kubernetes infrastructure components required by all backend services, including PostgreSQL, Redis/Valkey, S3/R2 storage, and initial Cloudflare Tunnel for Morgan agent access. This task establishes the shared environment for subsequent service deployments.

## Task Context
- Agent owner: Bolt
- Stack: Kubernetes/Helm
- Priority: high
- Dependencies: None

## Implementation Plan
1. Create 'databases' and 'sigma1' Kubernetes namespaces. 2. Deploy CloudNative-PG operator and a single-instance 'sigma1-postgres' cluster with 'sigma1' database and 'sigma1_user'. Ensure persistent storage is configured (50Gi). 3. Deploy Redis operator and a single-instance 'sigma1-valkey' cluster using 'valkey/valkey:7.2-alpine'. 4. Provision an S3/R2 bucket (e.g., 'sigma1-assets') via Cloudflare Terraform or equivalent, and generate necessary access credentials. 5. Set up a basic Cloudflare Tunnel for the 'morgan' service, pointing to a placeholder service or internal IP for initial connectivity. 6. Create a 'sigma1-infra-endpoints' ConfigMap in the 'sigma1' namespace, containing connection strings and credentials for PostgreSQL, Redis, and S3/R2, formatted for easy consumption by other services (e.g., `POSTGRES_URL`, `REDIS_URL`, `S3_BUCKET_NAME`, `S3_ACCESS_KEY_ID`, `S3_SECRET_ACCESS_KEY`).

## Acceptance Criteria
1. Verify 'databases' and 'sigma1' namespaces exist. 2. Confirm 'sigma1-postgres' and 'sigma1-valkey' pods are running and healthy in the 'databases' namespace. 3. Connect to PostgreSQL and Redis instances to verify accessibility and basic functionality. 4. Confirm S3/R2 bucket exists and credentials allow read/write access. 5. Verify 'sigma1-infra-endpoints' ConfigMap exists in 'sigma1' namespace and contains correct, accessible connection details for all provisioned infrastructure. 6. Confirm Cloudflare Tunnel is active and reachable from the internet (e.g., via `cloudflared tunnel status`).

## Subtasks
- Implement Core Infrastructure Setup (Bolt - Kubernetes/Helm): Provision the foundational Kubernetes infrastructure components required by all backend services, including PostgreSQL, Redis/Valkey, S3/R2 storage, and initial Cloudflare Tunnel for Morgan agent access. This task establishes the shared environment for subsequent service deployments.

## Deliverables
- Update the relevant code, configuration, and tests.
- Keep artifacts aligned with the acceptance criteria.
- Document blockers or assumptions in your final summary.