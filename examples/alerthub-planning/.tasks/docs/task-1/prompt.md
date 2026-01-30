# Implementation Prompt for Task 1

## Context
You are implementing "Infrastructure Setup (Bolt - Kubernetes)" for the AlertHub notification platform.

## PRD Reference
See `../../prd.md` for full requirements.

## Task Requirements
Deploy foundational infrastructure including PostgreSQL, Redis/Valkey, Kafka, MongoDB, RabbitMQ, and SeaweedFS using Kubernetes operators. This is the foundation for all other services.

## Implementation Details
Create Kubernetes namespace structure, deploy CloudNative-PG for PostgreSQL, Redis operator for Valkey, Strimzi for Kafka, Percona for MongoDB, RabbitMQ cluster operator, and SeaweedFS for object storage. Configure network policies, resource limits, and persistent volumes.

## Dependencies
This task has no dependencies and can be started immediately.

## Testing Requirements
All database operators report healthy status, databases are accessible via cluster DNS, persistent volumes are bound, and connection tests pass from within cluster

## Decision Points to Address

The following decisions need to be made during implementation:

### d1: Single namespace vs multiple namespaces for different infrastructure types
**Category**: architecture | **Constraint**: open

Options:
1. single alerthub namespace
2. separate namespaces by type (databases, messaging, storage)

Document your choice and rationale in the implementation.

### d2: Storage class selection for persistent volumes
**Category**: performance | **Constraint**: soft

Options:
1. default storage class
2. fast SSD storage class for databases

Document your choice and rationale in the implementation.


## Deliverables
1. Source code implementing the requirements
2. Unit tests with >80% coverage
3. Integration tests for external interfaces
4. Documentation updates as needed
5. Decision point resolutions documented

## Notes
- Follow project coding standards
- Use Effect TypeScript patterns where applicable
- Ensure proper error handling and logging
