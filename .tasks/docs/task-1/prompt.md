Implement task 1: Provision Core Infrastructure (Bolt - Kubernetes/Helm)

## Goal
Set up all foundational infrastructure for Sigma-1, including PostgreSQL, Redis/Valkey, S3/R2, Signal-CLI, and required ConfigMaps for service connection strings. This enables all backend and frontend services to connect to their dependencies.

## Task Context
- Agent owner: bolt
- Stack: Kubernetes/Helm
- Priority: high
- Dependencies: None

## Implementation Plan
{"steps": ["Create Kubernetes namespaces: databases, sigma1, openclaw, social, web, etc.", "Deploy CloudNative-PG PostgreSQL cluster (single instance, 50Gi, schemas: rms, crm, finance, audit, public)", "Deploy Redis/Valkey using Opstree operator (single instance)", "Provision S3/R2 bucket for object storage (product images, event photos)", "Deploy Signal-CLI as a sidecar or separate pod for Morgan agent integration", "Create ConfigMap 'sigma1-infra-endpoints' aggregating connection strings for all services (POSTGRES_URL, REDIS_URL, S3_URL, SIGNALCLI_URL, etc.)", "Provision secrets for API keys (Stripe, OpenCorporates, LinkedIn, Google, etc.)", "Document all endpoints and secret references for downstream services"]}

## Acceptance Criteria
Verify all pods (PostgreSQL, Redis, S3/R2, Signal-CLI) are running and healthy. Confirm ConfigMap 'sigma1-infra-endpoints' is present and contains valid connection strings. All secrets are accessible by service accounts.

## Subtasks
- Create Kubernetes namespaces for Sigma-1 platform: Create all required Kubernetes namespaces that will host the various infrastructure components and application services: databases, sigma1, openclaw, social, web. Apply standard labels and annotations for organizational tracking.
- Deploy CloudNative-PG PostgreSQL cluster with multi-schema setup: Deploy a single-instance CloudNative-PG PostgreSQL cluster in the databases namespace with 50Gi storage. Configure the database with five schemas: rms, crm, finance, audit, and public. Create appropriate roles and grant schema-level permissions.
- Deploy Redis/Valkey instance via Opstree operator: Deploy a single-instance Redis/Valkey cache using the Opstree Redis operator in the databases namespace. Configure it for session storage, caching, and rate limiting use cases across Sigma-1 services.
- Provision S3/R2 object storage bucket and access credentials: Create an S3-compatible object storage bucket (Cloudflare R2 or AWS S3 based on decision point dp-3) for product images, event photos, and other binary assets. Generate access keys and store them as Kubernetes secrets.
- Deploy Signal-CLI as a standalone pod for Morgan agent integration: Deploy Signal-CLI as a standalone Deployment in the sigma1 namespace to enable the Morgan AI agent to send and receive Signal messages. Expose it via an internal ClusterIP service with a REST/JSON-RPC interface.
- Provision Kubernetes Secrets for external API keys: Create Kubernetes Secrets for all third-party API keys required by Sigma-1 services: Stripe, OpenCorporates, LinkedIn, Google APIs, and any other external integrations. Use a consistent naming convention and namespace placement.
- Create sigma1-infra-endpoints ConfigMap aggregating all connection strings: Create the central ConfigMap 'sigma1-infra-endpoints' in the sigma1 namespace that aggregates connection strings and endpoint URLs for all provisioned infrastructure (PostgreSQL, Redis, S3, Signal-CLI). All downstream services will reference this ConfigMap via envFrom.
- Validate full infrastructure stack end-to-end: Run a comprehensive validation of all provisioned infrastructure components to ensure they are healthy, interconnected, and ready for downstream service deployment. Verify cross-namespace access patterns and document the final state.

## Deliverables
- Update the relevant code, configuration, and tests.
- Keep artifacts aligned with the acceptance criteria.
- Document blockers or assumptions in your final summary.