## Provision Core Infrastructure (Bolt - Kubernetes/Helm)

### Objective
Provision all core infrastructure resources required for Sigma-1, including PostgreSQL, Redis/Valkey, S3/R2, Signal-CLI, and external API secrets. Aggregate all service endpoints and credentials into a ConfigMap for downstream consumption.

### Ownership
- Agent: Bolt
- Stack: Kubernetes/Helm
- Priority: high
- Status: pending
- Dependencies: None

### Implementation Details
{"steps": ["Create Kubernetes namespaces: databases, sigma1, openclaw, social, web, etc.", "Deploy CloudNative-PG PostgreSQL cluster (single instance, 50Gi, schemas: rms, crm, finance, audit, public)", "Deploy Redis/Valkey using Opstree operator (single instance)", "Provision S3/R2 buckets for product images and social photos; configure access keys as Kubernetes secrets", "Deploy Signal-CLI as a sidecar or separate pod in openclaw namespace", "Store API credentials for Stripe, OpenCorporates, LinkedIn, Google Reviews, Instagram, Facebook as Kubernetes secrets", "Create a ConfigMap named sigma1-infra-endpoints with connection strings and service URLs for all provisioned resources (POSTGRES_URL, REDIS_URL, S3_URL, SIGNALCLI_URL, etc.)", "Document all endpoints and secret references for downstream service consumption"]}

### Subtasks
- [ ] Create Kubernetes namespaces and RBAC foundation: Create all required Kubernetes namespaces (databases, sigma1, openclaw, social, web) and configure basic RBAC service accounts so that downstream resources can be deployed into the correct namespace with appropriate permissions.
- [ ] Deploy CloudNative-PG PostgreSQL cluster with schemas: Deploy a single-instance CloudNative-PG PostgreSQL cluster in the databases namespace with 50Gi storage, and create the required schemas: rms, crm, finance, audit, public.
- [ ] Deploy Redis/Valkey via Opstree operator: Deploy a single-instance Redis/Valkey cache using the Opstree Redis operator in the databases namespace.
- [ ] Provision S3/R2 buckets and create access key secrets: Provision S3/R2 object storage buckets for product images and social photos, and store the access keys as Kubernetes secrets in the databases namespace.
- [ ] Deploy Signal-CLI pod in openclaw namespace: Deploy Signal-CLI as a standalone pod (or Deployment) in the openclaw namespace to serve as the Signal messaging relay for the Morgan agent.
- [ ] Create external API credential secrets: Create Kubernetes secrets for all external API credentials: Stripe, OpenCorporates, LinkedIn, Google Reviews, Instagram, and Facebook.
- [ ] Create sigma1-infra-endpoints ConfigMap aggregating all service endpoints: Create the sigma1-infra-endpoints ConfigMap in the databases namespace, aggregating connection strings and service URLs from all provisioned infrastructure resources.
- [ ] Validate end-to-end infrastructure connectivity: Run a comprehensive validation suite to ensure all provisioned infrastructure components are reachable, secrets are accessible, and the ConfigMap is correctly populated and consumable by downstream services.