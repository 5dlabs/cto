Implement task 1: Provision Core Infrastructure (Bolt - Kubernetes/Helm)

## Goal
Set up the foundational infrastructure for Sigma-1, including PostgreSQL, Redis/Valkey, S3/R2, Signal-CLI, ElevenLabs, Twilio, and required namespaces. Aggregate all service endpoints and credentials into a ConfigMap for downstream consumption.

## Task Context
- Agent owner: bolt
- Stack: Kubernetes/Helm
- Priority: high
- Dependencies: None

## Implementation Plan
{"steps": ["Create Kubernetes namespaces: databases, sigma1, openclaw, etc.", "Deploy CloudNative-PG PostgreSQL cluster (single instance, 50Gi, schemas: rms, crm, finance, audit, public)", "Deploy Redis/Valkey using operator (v7.2-alpine)", "Provision S3/R2 buckets for product images and event photos", "Deploy Signal-CLI as a sidecar or separate pod for Morgan agent", "Configure external service secrets for ElevenLabs, Twilio, Stripe, OpenCorporates, LinkedIn, Google Reviews, and credit APIs", "Create a ConfigMap named sigma1-infra-endpoints with connection strings and service URLs for all provisioned resources", "Document all endpoints and credentials for use by backend and frontend services"]}

## Acceptance Criteria
Verify all pods are running and healthy; confirm ConfigMap 'sigma1-infra-endpoints' contains valid connection strings for PostgreSQL, Redis, S3/R2, Signal-CLI, ElevenLabs, Twilio, and all external APIs; test connectivity from a test pod to each service endpoint.

## Subtasks
- Create Kubernetes namespaces and RBAC foundations: Create all required Kubernetes namespaces (databases, sigma1, openclaw, etc.) and set up basic RBAC ServiceAccounts for each namespace so downstream deployments have appropriate permissions.
- Deploy CloudNative-PG PostgreSQL cluster with schema initialization: Deploy a single-instance CloudNative-PG PostgreSQL cluster in the databases namespace with 50Gi storage and initialize schemas: rms, crm, finance, audit, public.
- Deploy Redis/Valkey instance via operator: Deploy a single-replica Redis-compatible instance (Valkey v7.2-alpine) using the Redis operator in the databases namespace.
- Provision S3/R2 buckets and access credentials: Create S3-compatible object storage buckets for product images and event photos, and store access credentials as Kubernetes Secrets.
- Deploy Signal-CLI pod for Morgan agent: Deploy Signal-CLI as a standalone pod (or sidecar-ready deployment) in the sigma1 namespace, configured with a registered Signal account for the Morgan agent.
- Create external service secrets for third-party APIs: Create Kubernetes Secrets in the sigma1 namespace for all external API credentials: ElevenLabs, Twilio, Stripe, OpenCorporates, LinkedIn, Google Reviews, and credit bureau APIs.
- Assemble sigma1-infra-endpoints ConfigMap: Create the sigma1-infra-endpoints ConfigMap aggregating all connection strings, service URLs, bucket names, and secret references from all provisioned infrastructure components.
- Validate end-to-end connectivity from test pod to all services: Deploy a test pod in the sigma1 namespace and validate connectivity to every provisioned service: PostgreSQL, Redis, S3/R2, Signal-CLI, and verify external API secret availability.

## Deliverables
- Update the relevant code, configuration, and tests.
- Keep artifacts aligned with the acceptance criteria.
- Document blockers or assumptions in your final summary.