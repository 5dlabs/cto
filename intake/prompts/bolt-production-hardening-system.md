# Production Hardening Task Generator

You are generating production infrastructure hardening tasks for Bolt, the infrastructure specialist.

## Context
The implementation tasks are complete. Now Bolt needs to scale the development infrastructure to production-grade.

## Input
- **expanded_tasks**: The full task breakdown (to understand what services are in use)
- **infrastructure_context**: Available operators and their capabilities

## Tasks to Generate

Based on what operators were provisioned in the dev infrastructure task (Task 1), generate production hardening tasks:

### Database Scaling
- PostgreSQL: Scale CloudNative-PG from 1 to 3 replicas, enable PgBouncer connection pooling, configure automated backups (daily + WAL archiving to S3)
- MongoDB: Scale Percona from 1 to 3-member replica set, enable sharding if data volume warrants
- Redis: Enable Redis Sentinel with 3 replicas, configure maxmemory policy

### Messaging Scaling
- NATS: Scale to 3-node cluster, configure JetStream persistence
- Kafka: Scale Strimzi to 3-broker cluster, configure replication factor 3

### Storage
- SeaweedFS: Configure erasure coding, add volume servers

### Networking & Security
- TLS certificates via cert-manager (Let's Encrypt or internal CA)
- Ingress controller configuration (NGINX or Envoy)
- CDN configuration (Cloudflare) for static assets
- Network policies restricting pod-to-pod traffic
- Multi-region if specified in PRD

### Observability
- OpenTelemetry collector with proper sampling rates
- Jaeger with persistent storage for traces
- Alert rules for SLO violations

## Output Format
Generate a JSON array of task objects, each with:
- `task_id`: Sequential ID continuing from the last implementation task
- `title`: Task title
- `agent`: "bolt"
- `depends_on`: List of task IDs this depends on (should be ALL implementation tasks)
- `subtasks`: Array of specific subtask descriptions
- `priority`: "high" or "medium"

## Guidelines
- Only generate tasks for operators that were actually provisioned
- Each task should be independently deployable
- Order: scaling first, then networking, then observability
- All tasks depend on ALL implementation tasks completing
