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
1. Create `databases` and `sigma1` Kubernetes namespaces.2. Deploy CloudNative-PG operator and provision a `sigma1-postgres` Cluster with a `sigma1` database and `sigma1_user` owner, 50Gi storage, and single instance for dev. Ensure multiple schemas (rms, crm, finance, audit, public) can be created within this database.3. Deploy Redis/Valkey operator and provision a `sigma1-valkey` Redis instance (Valkey 7.2-alpine) with single instance for dev.4. Configure S3/R2 bucket access (e.g., via Kubernetes secrets for credentials) for image storage.5. Create a `sigma1-infra-endpoints` ConfigMap in the `sigma1` namespace, containing connection strings for PostgreSQL and Redis/Valkey, following the pattern `{OPERATOR}_{INSTANCE}_URL` (e.g., `POSTGRES_SIGMA1_POSTGRES_URL`, `REDIS_SIGMA1_VALKEY_URL`).6. Ensure basic network policies are in place to allow internal service communication.

### Subtasks
- [ ] Create core Kubernetes namespaces: Create the 'databases' and 'sigma1' Kubernetes namespaces to logically separate infrastructure components and application services.
- [ ] Deploy CloudNative-PG operator and PostgreSQL cluster: Deploy the CloudNative-PG operator and provision a single-instance PostgreSQL cluster named 'sigma1-postgres' with a 'sigma1' database and 'sigma1_user' owner, 50Gi storage, within the 'databases' namespace.
- [ ] Deploy Redis/Valkey operator and instance: Deploy the Redis/Valkey operator and provision a single-instance Redis/Valkey instance named 'sigma1-valkey' (Valkey 7.2-alpine) within the 'databases' namespace.
- [ ] Configure S3/R2 bucket access credentials: Create Kubernetes secrets in the 'sigma1' namespace to securely store S3/R2 bucket access credentials for image storage.
- [ ] Create sigma1-infra-endpoints ConfigMap: Create a ConfigMap named 'sigma1-infra-endpoints' in the 'sigma1' namespace, containing connection strings for the deployed PostgreSQL and Redis/Valkey instances.
- [ ] Implement basic internal network policies: Apply basic network policies within the 'sigma1' namespace to allow internal service communication, ensuring services can reach PostgreSQL and Redis/Valkey.