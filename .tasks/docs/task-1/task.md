## Bootstrap Core Infrastructure (Bolt - Kubernetes/Helm)

### Objective
Provision all foundational infrastructure for Sigma-1, including PostgreSQL, Redis/Valkey, S3/R2, Signal-CLI, and required ConfigMaps for service connection strings. This enables all backend and frontend services to connect to their dependencies.

### Ownership
- Agent: bolt
- Stack: Kubernetes/Helm
- Priority: high
- Status: pending
- Dependencies: None

### Implementation Details
{"steps": ["Create Kubernetes namespaces: databases, sigma1, openclaw, social, web, etc.", "Deploy CloudNative-PG operator and provision a single-replica PostgreSQL 16 cluster (schemas: rms, crm, finance, audit, public)", "Deploy Redis/Valkey using the opstreelabs operator in the databases namespace", "Provision S3/R2 buckets for product images and event photos; expose endpoints via ConfigMap", "Deploy Signal-CLI as a sidecar or separate pod for Morgan agent integration", "Create ConfigMap 'sigma1-infra-endpoints' aggregating connection strings for all services (POSTGRES_URL, REDIS_URL, S3_URL, SIGNAL_CLI_URL, etc.)", "Provision secrets for API keys (Stripe, OpenCorporates, LinkedIn, Google, etc.) in the appropriate namespaces", "Document all endpoints and secret names for downstream consumption"]}

### Subtasks
- [ ] Create Kubernetes namespaces and RBAC foundation: Create all required Kubernetes namespaces for the Sigma-1 platform (databases, sigma1, openclaw, social, web) and configure baseline RBAC ServiceAccounts and RoleBindings so that each namespace's workloads can access their designated secrets and ConfigMaps.
- [ ] Deploy PostgreSQL 16 via CloudNative-PG operator with multi-schema setup: Deploy a single-replica PostgreSQL 16 cluster in the databases namespace using the CloudNative-PG operator, then create the required schemas (rms, crm, finance, audit, public) and initial roles for each downstream service.
- [ ] Deploy Redis/Valkey via opstreelabs operator: Deploy a single-replica Redis/Valkey instance in the databases namespace using the opstreelabs Redis operator, configured for caching, rate limiting, and session storage.
- [ ] Provision S3/R2 buckets for product images and event photos: Create S3-compatible object storage buckets for product images and event photos, configure access credentials, and expose bucket endpoints for downstream services.
- [ ] Deploy Signal-CLI for Morgan agent integration: Deploy Signal-CLI as a standalone pod in the sigma1 namespace with REST API access, enabling the Morgan agent to send and receive Signal messages.
- [ ] Provision third-party API secrets across namespaces: Create Kubernetes Secrets for all third-party API keys (Stripe, OpenCorporates, LinkedIn, Google, etc.) in their respective namespaces so downstream services can consume them securely.
- [ ] Create sigma1-infra-endpoints ConfigMap aggregating all connection strings: Create the central 'sigma1-infra-endpoints' ConfigMap in the sigma1 namespace that aggregates all infrastructure connection strings and endpoints, enabling downstream services to consume them via envFrom.
- [ ] Validate end-to-end infrastructure connectivity from a test pod: Deploy a temporary test pod that loads connection details from the sigma1-infra-endpoints ConfigMap and validates connectivity to PostgreSQL, Redis, S3/R2, and Signal-CLI.