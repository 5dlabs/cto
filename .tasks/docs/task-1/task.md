## Provision Core Infrastructure (Bolt - Kubernetes/Helm)

### Objective
Set up the foundational Kubernetes infrastructure including namespaces, PostgreSQL, Redis/Valkey, and S3/R2 storage. This task establishes the persistent data and caching layers required by all backend services.

### Ownership
- Agent: bolt
- Stack: Kubernetes/Helm
- Priority: high
- Status: pending
- Dependencies: None

### Implementation Details
1. Create `databases` and `sigma1` Kubernetes namespaces.
2. Deploy CloudNative-PG operator and provision a `sigma1-postgres` Cluster with a `sigma1` database and `sigma1_user` owner, 50Gi storage, and single instance for dev. Ensure multiple schemas (rms, crm, finance, audit, public) can be created within this database.
3. Deploy Redis/Valkey operator and provision a `sigma1-valkey` Redis instance (Valkey 7.2-alpine) with single instance for dev.
4. Configure S3/R2 bucket access (e.g., via Kubernetes secrets for credentials) for image storage.
5. Create a `sigma1-infra-endpoints` ConfigMap in the `sigma1` namespace, containing connection strings for PostgreSQL and Redis/Valkey, following the pattern `{OPERATOR}_{INSTANCE}_URL` (e.g., `POSTGRES_SIGMA1_POSTGRES_URL`, `REDIS_SIGMA1_VALKEY_URL`).
6. Ensure basic network policies are in place to allow internal service communication.

### Subtasks
- [ ] Implement Provision Core Infrastructure (Bolt - Kubernetes/Helm): Set up the foundational Kubernetes infrastructure including namespaces, PostgreSQL, Redis/Valkey, and S3/R2 storage. This task establishes the persistent data and caching layers required by all backend services.