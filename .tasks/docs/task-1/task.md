## Provision Core Infrastructure (Bolt - Kubernetes/Helm)

### Objective
Provision all foundational infrastructure for Sigma-1, including PostgreSQL, Redis/Valkey, S3/R2, Signal-CLI, and required ConfigMaps for service connection strings. This enables all backend and frontend services to connect to their dependencies.

### Ownership
- Agent: bolt
- Stack: Kubernetes/Helm
- Priority: high
- Status: pending
- Dependencies: None

### Implementation Details
{"steps":["Create Kubernetes namespaces: databases, sigma1, openclaw, social, web, etc.","Deploy CloudNative-PG operator and provision a single-replica PostgreSQL 16 cluster with schemas for rms, crm, finance, audit, public.","Deploy Redis/Valkey using the opstreelabs operator.","Provision S3/R2 buckets for product images and event photos.","Deploy Signal-CLI as a sidecar or separate pod for Morgan agent.","Create ConfigMaps named sigma1-infra-endpoints in each namespace, aggregating connection strings for all services (POSTGRES_URL, REDIS_URL, S3_URL, SIGNAL_CLI_URL, etc.).","Provision secrets for API keys (Stripe, OpenCorporates, LinkedIn, Google, etc.) in Kubernetes secrets.","Document all endpoints and secret names for downstream service consumption."]}

### Subtasks
- [ ] Create Kubernetes namespaces for all Sigma-1 service domains: Create all required Kubernetes namespaces (databases, sigma1, openclaw, social, web, and any others referenced by the architecture) with appropriate labels and annotations for service discovery and RBAC scoping.
- [ ] Deploy PostgreSQL 16 via CloudNative-PG with multi-schema setup: Deploy a single-replica PostgreSQL 16 cluster using the CloudNative-PG operator in the databases namespace, and create schemas for rms, crm, finance, audit, and public.
- [ ] Deploy Redis/Valkey via opstreelabs operator: Deploy a single-replica Redis/Valkey instance using the opstreelabs Redis operator in the databases namespace for caching, rate limiting, and session storage.
- [ ] Provision S3/R2 buckets for product images and event photos: Create and configure S3-compatible object storage buckets (Cloudflare R2 or AWS S3) for product images and event photos, including access credentials and CORS policies.
- [ ] Deploy Signal-CLI as a standalone pod for Morgan agent: Deploy Signal-CLI as a dedicated Kubernetes Deployment in the sigma1 namespace, configured to receive and send Signal messages for the Morgan AI agent.
- [ ] Create sigma1-infra-endpoints ConfigMap across all namespaces: Create the `sigma1-infra-endpoints` ConfigMap in each service namespace, aggregating connection strings for PostgreSQL, Redis, S3, Signal-CLI, and all other infrastructure endpoints.
- [ ] Provision Kubernetes Secrets for third-party API keys: Create Kubernetes Secrets in the appropriate namespaces for all third-party API keys required by Sigma-1 services (Stripe, OpenCorporates, LinkedIn, Google, etc.).
- [ ] Validate end-to-end infrastructure connectivity from all namespaces: Run connectivity tests from each service namespace to verify that all infrastructure components (PostgreSQL, Redis, S3, Signal-CLI) are reachable using the ConfigMap and Secret values.