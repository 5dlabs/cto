Implement task 1: Provision Core Infrastructure (Bolt - Kubernetes/Helm)

## Goal
Provision all foundational infrastructure for Sigma-1, including PostgreSQL, Redis/Valkey, S3/R2, Signal-CLI, ElevenLabs, Twilio, and create a ConfigMap aggregating all service endpoints for downstream consumption.

## Task Context
- Agent owner: Bolt
- Stack: Kubernetes/Helm
- Priority: high
- Dependencies: None

## Implementation Plan
{"steps": ["Create Kubernetes namespaces: databases, sigma1, openclaw, social, web, etc.", "Deploy CloudNative-PG PostgreSQL cluster (single instance, 50Gi, schemas: rms, crm, finance, audit, public)", "Deploy Redis/Valkey using Opstree operator (single instance, 7.2-alpine)", "Provision S3/R2 buckets for product images and event photos, expose endpoints via ConfigMap", "Deploy Signal-CLI as a sidecar or separate pod in openclaw namespace", "Configure external service secrets for ElevenLabs, Twilio, OpenCorporates, LinkedIn, Google Reviews, Stripe, and store in Kubernetes secrets", "Create a ConfigMap named 'sigma1-infra-endpoints' with connection strings and API URLs for all services (POSTGRES_URL, REDIS_URL, S3_URL, SIGNAL_CLI_URL, etc.)", "Deploy Cloudflare Tunnel for Morgan agent ingress", "Document all endpoints and secret keys for downstream service consumption"]}

## Acceptance Criteria
All pods are running and healthy; ConfigMap 'sigma1-infra-endpoints' is available in all namespaces; all secrets are mounted and accessible; test connections to PostgreSQL, Redis, S3/R2, and Signal-CLI succeed.

## Subtasks
- Create Kubernetes namespaces and base RBAC configuration: Create all required Kubernetes namespaces (databases, sigma1, openclaw, social, web) and configure basic RBAC ServiceAccounts so downstream deployments can reference secrets and ConfigMaps across namespaces.
- Deploy CloudNative-PG PostgreSQL cluster with multi-schema setup: Deploy a single-instance CloudNative-PG PostgreSQL cluster in the databases namespace with 50Gi storage, and initialize the required schemas: rms, crm, finance, audit, and public.
- Deploy Redis/Valkey using Opstree operator: Deploy a single-instance Redis/Valkey cache using the Opstree Redis operator in the databases namespace, configured with 7.2-alpine image.
- Provision S3/R2 buckets and configure access credentials: Create S3/R2 buckets for product images and event photos, configure access keys, and store credentials as Kubernetes secrets.
- Deploy Signal-CLI pod in openclaw namespace: Deploy Signal-CLI as a standalone pod or Deployment in the openclaw namespace, configured for Morgan agent messaging integration.
- Create external service secrets for all third-party integrations: Create Kubernetes Secrets containing API keys and credentials for ElevenLabs, Twilio, OpenCorporates, LinkedIn, Google Reviews, and Stripe.
- Deploy Cloudflare Tunnel for Morgan agent ingress: Deploy a Cloudflare Tunnel (cloudflared) in the cluster to provide secure external ingress for the Morgan agent without exposing a public IP.
- Create aggregated 'sigma1-infra-endpoints' ConfigMap: Create the 'sigma1-infra-endpoints' ConfigMap that aggregates all service connection strings and API URLs, and make it available across all namespaces.
- Document all infrastructure endpoints, secrets, and access patterns: Create comprehensive documentation of all provisioned infrastructure, including endpoint URLs, secret key names, access patterns, and instructions for downstream service consumption.

## Deliverables
- Update the relevant code, configuration, and tests.
- Keep artifacts aligned with the acceptance criteria.
- Document blockers or assumptions in your final summary.