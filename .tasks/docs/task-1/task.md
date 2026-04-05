## Bootstrap Core Infrastructure (Bolt - Kubernetes/Helm)

### Objective
Provision all shared infrastructure for the Sigma-1 platform: namespace creation, CloudNative-PG PostgreSQL cluster with multi-schema bootstrap, Valkey instance via existing Opstree Redis operator, Cloudflare R2 bucket configuration, External Secrets references, and a sigma1-infra-endpoints ConfigMap aggregating all connection strings. This is the foundational task that all backend services depend on.

### Ownership
- Agent: bolt
- Stack: Kubernetes/Helm
- Priority: high
- Status: pending
- Dependencies: None

### Implementation Details
1. Create Kubernetes namespace `sigma1` for application workloads.
2. Create CloudNative-PG `Cluster` CR in `sigma1` namespace:
   - `name: sigma1-postgres`, PostgreSQL 16, single instance (dev), 50Gi storage
   - Bootstrap initdb with database `sigma1`, owner `sigma1_user`
   - Post-init SQL to create schemas: `rms`, `crm`, `finance`, `vetting`, `social`, `audit`, `public`
   - Create per-service Postgres users with schema-scoped permissions:
     - `sigma1_catalog` → USAGE/ALL on `public` schema
     - `sigma1_rms` → USAGE/ALL on `rms` schema
     - `sigma1_finance` → USAGE/ALL on `finance` schema
     - `sigma1_vetting` → USAGE/ALL on `vetting` schema
     - `sigma1_social` → USAGE/ALL on `social` schema
     - `sigma1_audit` → USAGE/ALL on `audit` schema (all services get INSERT)
   - Enforce no cross-schema JOIN capability via GRANT restrictions (per D5 recommendation)
3. Create Valkey instance via existing Opstree Redis operator:
   - `Redis` CR: name `sigma1-valkey`, namespace `sigma1`, image `valkey/valkey:7.2-alpine`
   - Single replica for dev
4. Configure Cloudflare R2 bucket `sigma1-assets` via existing Cloudflare operator or Terraform:
   - Sub-prefixes: `products/`, `social/`, `portfolio/`
   - Generate R2 API credentials, store as ExternalSecret
5. Create ExternalSecret CRs for all third-party API keys (placeholders):
   - `sigma1-stripe-keys` (Stripe publishable + secret)
   - `sigma1-opencorporates-key`
   - `sigma1-social-api-keys` (Instagram, LinkedIn, Facebook tokens)
   - `sigma1-elevenlabs-key`
   - `sigma1-twilio-keys`
   - `sigma1-openai-key`
   - `sigma1-google-calendar-creds`
6. Create `sigma1-infra-endpoints` ConfigMap:
   - `CNPG_SIGMA1_POSTGRES_URL`: pointing to CNPG cluster service
   - `REDIS_SIGMA1_VALKEY_URL`: pointing to Valkey service
   - `R2_SIGMA1_ASSETS_ENDPOINT`: R2 endpoint URL
   - `R2_SIGMA1_ASSETS_BUCKET`: `sigma1-assets`
   - Per-service database URLs with schema-scoped credentials
7. Create shared RBAC role definitions as a ConfigMap `sigma1-rbac-roles`:
   - JSON schema defining roles: `admin`, `operator`, `morgan-agent`, `readonly`
   - Permission matrix per role (per D10 recommendation: shared schema, per-service implementation)
8. Create Cilium NetworkPolicy CRs:
   - Allow `sigma1` namespace pods to reach `sigma1-postgres` and `sigma1-valkey`
   - Allow Morgan pod to reach all backend services
   - Deny all other cross-namespace traffic by default
9. Deploy ServiceMonitor CRs for Prometheus scraping of all future services in `sigma1` namespace.
10. Validate all resources are Ready: CNPG cluster reporting `healthy`, Valkey pod running, ConfigMap populated.

### Subtasks
- [ ] Create sigma1 Kubernetes namespace: Create the sigma1 namespace with appropriate labels for network policy selection, monitoring, and resource quota boundaries.
- [ ] Create CloudNative-PG Cluster CR with initdb bootstrap: Deploy the CloudNative-PG Cluster custom resource for PostgreSQL 16 with initdb bootstrap creating the sigma1 database owned by sigma1_user.
- [ ] Author CNPG post-init SQL for schemas and scoped users: Write the post-init SQL that creates all 7 schemas, 6+ per-service users, and fine-grained GRANT restrictions preventing cross-schema JOINs. Integrate this SQL into the CNPG Cluster CR's postInitApplicationSQL or as a ConfigMap-mounted script.
- [ ] Create Valkey instance via Opstree Redis operator CR: Deploy a Valkey 7.2 instance using the existing Opstree Redis operator by creating a Redis custom resource in the sigma1 namespace.
- [ ] Configure Cloudflare R2 bucket and credentials: Provision the sigma1-assets R2 bucket with the required sub-prefix structure and store R2 API credentials as a Kubernetes Secret (or ExternalSecret).
- [ ] Create ExternalSecret CRs for third-party API keys: Create ExternalSecret custom resources for all third-party service API keys as placeholders, referencing the chosen external secrets backend.
- [ ] Create sigma1-infra-endpoints ConfigMap: Create the central ConfigMap that aggregates all infrastructure connection strings and endpoints for consumption by backend services via envFrom.
- [ ] Create sigma1-rbac-roles ConfigMap: Create the shared RBAC role definitions ConfigMap that defines the application-level role/permission matrix for all services to consume.
- [ ] Create Cilium NetworkPolicy CRs: Deploy Cilium CiliumNetworkPolicy custom resources to enforce network segmentation: allow sigma1 pods to reach PostgreSQL and Valkey, allow Morgan to reach all backend services, deny cross-namespace traffic by default.
- [ ] Deploy ServiceMonitor CRs for Prometheus scraping: Create ServiceMonitor custom resources so Prometheus automatically discovers and scrapes metrics from all services deployed in the sigma1 namespace.
- [ ] Validate all infrastructure resources are healthy and connected: Run a comprehensive validation suite to confirm every provisioned resource is ready, accessible, and correctly configured — CNPG cluster healthy, schema isolation enforced, Valkey responding, ConfigMap populated, ExternalSecrets synced, NetworkPolicies active.