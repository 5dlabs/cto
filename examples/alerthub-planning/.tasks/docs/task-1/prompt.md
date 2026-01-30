# Task 1: Infrastructure Setup (Bolt - Kubernetes)

**Agent**: bolt | **Language**: yaml

## Role

You are a DevOps Engineer specializing in Kubernetes implementing Task 1.

## Goal

Deploy foundational infrastructure including PostgreSQL, Redis/Valkey, Kafka, MongoDB, RabbitMQ, and SeaweedFS using Kubernetes operators. This is the foundation for all other services.

## Requirements

Create Kubernetes namespace structure, deploy CloudNative-PG for PostgreSQL, Redis operator for Valkey, Strimzi for Kafka, Percona for MongoDB, RabbitMQ cluster operator, and SeaweedFS for object storage. Configure network policies, resource limits, and persistent volumes.

## Acceptance Criteria

All database operators report healthy status, databases are accessible via cluster DNS, persistent volumes are bound, and connection tests pass from within cluster

## Constraints

- Match existing codebase patterns and style
- Create PR with atomic, well-described commits
- Include unit tests for new functionality
- PR title: `feat(task-1): Infrastructure Setup (Bolt - Kubernetes)`

## Decision Points

### d1: Single namespace vs multiple namespaces for different infrastructure types
**Category**: architecture | **Constraint**: open

Options:
1. single alerthub namespace
2. separate namespaces by type (databases, messaging, storage)

### d2: Storage class selection for persistent volumes
**Category**: performance | **Constraint**: soft

Options:
1. default storage class
2. fast SSD storage class for databases


## Resources

- PRD: `.tasks/docs/prd.md`

