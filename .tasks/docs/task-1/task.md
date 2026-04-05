## Provision Core Infrastructure (Bolt - Kubernetes/Helm)

### Objective
Provision all foundational infrastructure for Sigma-1, including PostgreSQL, Redis/Valkey, S3/R2, Signal-CLI, ElevenLabs, Twilio, and create a ConfigMap aggregating all service endpoints for downstream consumption.

### Ownership
- Agent: Bolt
- Stack: Kubernetes/Helm
- Priority: high
- Status: pending
- Dependencies: None

### Implementation Details
{"steps": ["Create Kubernetes namespaces: databases, sigma1, openclaw, social, web, etc.", "Deploy CloudNative-PG PostgreSQL cluster (single instance, 50Gi, schemas: rms, crm, finance, audit, public)", "Deploy Redis/Valkey using Opstree operator (single instance)", "Provision S3/R2 buckets for product images and event photos, expose endpoints via ConfigMap", "Deploy Signal-CLI as a sidecar or separate pod in openclaw namespace", "Configure external service secrets for ElevenLabs, Twilio, Stripe, OpenCorporates, LinkedIn, Google Reviews, and store in Kubernetes secrets", "Create a ConfigMap named sigma1-infra-endpoints with connection strings and API URLs for all services (POSTGRES_URL, REDIS_URL, S3_URL, SIGNAL_CLI_URL, etc.)", "Document all endpoints and secret keys for downstream service consumption."]}

### Subtasks
- [ ] Create Kubernetes namespaces and RBAC foundations: Create all required Kubernetes namespaces (databases, sigma1, openclaw, social, web) and apply baseline RBAC roles and service accounts for each namespace to enable subsequent deployments.
- [ ] Deploy CloudNative-PG PostgreSQL cluster with schema initialization: Deploy a single-instance CloudNative-PG PostgreSQL cluster in the databases namespace with 50Gi storage and initialize all required schemas (rms, crm, finance, audit, public).
- [ ] Deploy Redis/Valkey via Opstree operator: Deploy a single-instance Redis/Valkey instance using the Opstree Redis operator in the databases namespace for caching, rate limiting, and session storage.
- [ ] Provision S3/R2 buckets and configure access credentials: Provision S3-compatible object storage buckets for product images and event photos, and store access credentials and endpoint URLs as Kubernetes Secrets.
- [ ] Deploy Signal-CLI pod in openclaw namespace: Deploy Signal-CLI as a standalone pod (or deployment) in the openclaw namespace, configured for REST API access by downstream services.
- [ ] Create external service Kubernetes Secrets: Create Kubernetes Secrets for all external third-party API credentials: ElevenLabs, Twilio, Stripe, OpenCorporates, LinkedIn, and Google Reviews.
- [ ] Create sigma1-infra-endpoints ConfigMap aggregating all service endpoints: Create the central ConfigMap named sigma1-infra-endpoints in the sigma1 namespace, containing connection strings and API URLs for all provisioned services.
- [ ] Validate end-to-end infrastructure connectivity: Run a comprehensive connectivity test from a test pod to verify all provisioned infrastructure services are reachable and functional, and that the ConfigMap provides correct endpoints.