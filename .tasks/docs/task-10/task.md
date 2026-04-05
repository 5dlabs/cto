## Production Hardening: RBAC, Secrets Rotation, Audit Logging, Security (Bolt - Kubernetes/Helm)

### Objective
Implement production security hardening: service-to-service JWT token issuance and rotation via External Secrets, GDPR audit logging infrastructure, secret rotation policies, pod security standards, and security scanning CI integration. Ensures compliance with GDPR requirements and operational security posture.

### Ownership
- Agent: bolt
- Stack: Kubernetes/Helm
- Priority: high
- Status: pending
- Dependencies: 9

### Implementation Details
1. Service-to-service JWT tokens (per D6 recommendation):
   - Generate RSA-256 key pair, store private key as ExternalSecret `sigma1-jwt-signing-key`
   - Create deploy-time Job (or init container) that generates JWT service tokens for each service:
     - `equipment-catalog-token` with claims { sub: 'equipment-catalog', roles: ['service'] }
     - `rms-token` with claims { sub: 'rms', roles: ['service'] }
     - `finance-token` with claims { sub: 'finance', roles: ['service'] }
     - `vetting-token` with claims { sub: 'customer-vetting', roles: ['service'] }
     - `social-engine-token` with claims { sub: 'social-engine', roles: ['service'] }
     - `morgan-token` with claims { sub: 'morgan', roles: ['morgan-agent'] }
   - Tokens stored as Kubernetes Secrets, mounted as env vars
   - Token expiry: 90 days, with CronJob for rotation 30 days before expiry
   - Public key distributed via ConfigMap `sigma1-jwt-public-key` for verification by all services
2. Secret rotation policy:
   - Configure External Secrets operator refresh interval: 1 hour
   - Database password rotation via CNPG scheduled rotation (every 90 days)
   - R2 API key rotation: manual trigger via ExternalSecret refresh
   - Document rotation runbook in ConfigMap `sigma1-ops-runbooks`
3. GDPR audit logging infrastructure:
   - Create `audit` schema tables (migration in CNPG init):
     - `audit_log` table: id (UUID), service_name, action (enum: create/read/update/delete/export), entity_type, entity_id, actor_service, actor_user_id (nullable), timestamp, request_metadata (JSONB: IP, user-agent)
     - `data_export_requests` table: id, customer_id, requested_at, completed_at, export_url (R2 signed URL), expires_at
     - `data_deletion_requests` table: id, customer_id, requested_at, completed_at, services_purged (TEXT[])
   - All services must INSERT to audit_log for any operation touching customer data
   - Each service's shared-auth middleware (Rust) or RBAC middleware (Go/Node) enriched to auto-log on customer data access
4. GDPR data export endpoint (cross-service orchestration):
   - Create a CronJob or on-demand Job that:
     - Queries each service for customer data via their APIs
     - Aggregates into JSON export file
     - Uploads to R2 with signed URL (7-day expiry)
     - Records in data_export_requests
5. GDPR data deletion orchestration:
   - On deletion request, Job calls DELETE endpoints on each service:
     - Customer Vetting: DELETE /api/v1/vetting/:org_id
     - Finance: marks invoices as anonymized (retains for tax compliance, removes PII)
     - RMS: anonymizes customer data in opportunities/projects
     - Social Engine: removes any photos associated with customer
   - Records completion in data_deletion_requests with services_purged list
6. Pod Security Standards:
   - Apply `restricted` PodSecurity level to sigma1 namespace
   - All containers: runAsNonRoot: true, readOnlyRootFilesystem: true (with emptyDir for tmp)
   - Drop all capabilities, add only NET_BIND_SERVICE where needed
   - SecurityContext: allowPrivilegeEscalation: false
7. Container image security:
   - All images use distroless or alpine-slim base
   - Configure Kyverno or Gatekeeper policy: only allow images from approved registries
   - Image pull policy: Always (for mutable tags) or IfNotPresent (for SHA-pinned)
8. CI/CD security integration:
   - Add Cipher agent pipeline step definitions:
     - Semgrep rules for Rust, Go, TypeScript
     - Snyk/Dependabot configuration for dependency scanning
     - CodeQL workflow for Go and TypeScript
   - Merge blocker: critical/high severity findings
9. Rate limiting at ingress level:
   - Cloudflare WAF rules: rate limit public API endpoints (100 req/min per IP)
   - Bot protection on website
10. Monitoring alerts:
    - AlertManager rules:
      - Service down (0 ready pods) → critical
      - CNPG replica lag > 10s → warning
      - Error rate > 5% on any service → warning
      - Certificate/token expiry < 14 days → warning
      - Disk usage > 80% on PVCs → warning

### Subtasks
- [ ] Generate RSA-256 key pair and store as ExternalSecret for JWT signing: Create the RSA-256 key pair that will be used for service-to-service JWT token signing. Store the private key as an ExternalSecret `sigma1-jwt-signing-key` and distribute the public key via ConfigMap `sigma1-jwt-public-key` so all services can verify tokens.
- [ ] Create deploy-time Job for JWT service token generation for all 6 services: Implement a Kubernetes Job that runs at deploy time, reads the JWT signing private key, and generates service tokens for all 6 services (equipment-catalog, rms, finance, customer-vetting, social-engine, morgan) with appropriate claims and 90-day expiry, storing each as a Kubernetes Secret.
- [ ] Create CronJob for JWT token rotation with graceful rollover: Implement a Kubernetes CronJob that runs every 60 days (30 days before token expiry) to regenerate all service JWT tokens, update Secrets, and trigger rolling restarts of services to pick up new tokens without downtime.
- [ ] Configure secret rotation policies for ExternalSecrets, CNPG, and R2: Set up ExternalSecrets refresh intervals, CNPG database password rotation schedule, R2 API key rotation documentation, and create the operational runbook ConfigMap.
- [ ] Create GDPR audit logging database schema and migration: Define the `audit` schema with `audit_log`, `data_export_requests`, and `data_deletion_requests` tables as a CNPG init migration, ensuring all tables have proper indexes and constraints.
- [ ] Create GDPR data export orchestration Job: Implement a Kubernetes Job (triggered on-demand) that queries each service's API for customer data, aggregates results into a JSON file, uploads to R2 with a signed URL (7-day expiry), and records the export request in the database.
- [ ] Create GDPR data deletion orchestration Job: Implement a Kubernetes Job (triggered on-demand) that calls DELETE/anonymization endpoints on each service for a given customer, tracks completion per service, and records the deletion request in the database.
- [ ] Apply Pod Security Standards to sigma1 namespace: Enforce the 'restricted' PodSecurity level on the sigma1 namespace and update all Deployment/StatefulSet SecurityContexts to comply: runAsNonRoot, readOnlyRootFilesystem, drop all capabilities, disallow privilege escalation.
- [ ] Configure container image admission policy with Kyverno or Gatekeeper: Deploy an admission controller policy that restricts container images in the sigma1 namespace to approved registries only, and enforce image pull policies.
- [ ] Create CI/CD security scanning pipeline definitions (Semgrep, Snyk, CodeQL): Define CI/CD pipeline step configurations for static analysis (Semgrep), dependency scanning (Snyk/Dependabot), and code scanning (CodeQL) across Rust, Go, and TypeScript codebases, with merge blockers for critical/high findings.
- [ ] Configure Cloudflare WAF rate limiting and bot protection rules: Define Cloudflare WAF rules for rate limiting public API endpoints (100 req/min per IP) and bot protection on the website, expressed as Terraform resources or Cloudflare API configurations.
- [ ] Create AlertManager rules for service monitoring and alerting: Define Prometheus AlertManager rules for: service down (0 ready pods), CNPG replica lag, error rate spikes, certificate/token expiry warnings, and PVC disk usage warnings.