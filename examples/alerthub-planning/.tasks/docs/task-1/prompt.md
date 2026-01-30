# Task 1: Setup Infrastructure Components (Bolt - Kubernetes)

**Agent**: bolt | **Language**: yaml

## Role

You are a DevOps Engineer specializing in Kubernetes implementing Task 1.

## Goal

Provision core infrastructure services including PostgreSQL, Redis/Valkey, Kafka, MongoDB, RabbitMQ, and SeaweedFS using Kubernetes operators. This foundational task enables all backend services to have their required data stores and messaging infrastructure.

## Requirements

1. Deploy CloudNative-PG PostgreSQL cluster with alerthub database
2. Deploy Valkey/Redis for caching and rate limiting
3. Deploy Strimzi Kafka cluster for event streaming
4. Deploy Percona MongoDB for integration configs
5. Deploy RabbitMQ cluster for task queuing
6. Deploy SeaweedFS for object storage
7. Configure network policies for service isolation
8. Set up persistent volume claims
9. Create service accounts and RBAC
10. Verify all operators are healthy and ready

## Acceptance Criteria

All infrastructure CRDs are applied successfully, pods are in Running state, services are accessible from within cluster, health checks pass, and basic connectivity tests succeed (e.g., can connect to PostgreSQL, Redis responds to ping, Kafka topics can be created).

## Constraints

- Match existing codebase patterns
- Create PR with atomic commits
- Include unit tests
- PR title: `feat(task-1): Setup Infrastructure Components (Bolt - Kubernetes)`

## Decision Points

### d1: Should we use a single namespace or separate namespaces for different infrastructure types (databases, messaging, storage)?
**Category**: architecture | **Constraint**: soft

Options:
1. single-namespace
2. multi-namespace

### d2: What resource limits should be set for each infrastructure component in development vs production?
**Category**: performance | **Constraint**: escalation | ⚠️ **Requires Approval**

Options:
1. minimal-resources
2. production-like-resources


## Resources

- PRD: `.tasks/docs/prd.md`

