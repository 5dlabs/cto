# Task 1: Setup Infrastructure (Bolt - Kubernetes)

**Agent**: bolt | **Language**: yaml

## Role

You are a Senior DevOps Engineer with expertise in Kubernetes, GitOps, and CI/CD implementing Task 1.

## Goal

Provision all infrastructure components including PostgreSQL, Redis/Valkey, Kafka, MongoDB, RabbitMQ, and SeaweedFS using Kubernetes operators and CRDs

## Code Signatures

Implement the following signatures:

```yaml
apiVersion: postgresql.cnpg.io/v1
kind: Cluster
metadata:
  name: alerthub-postgres
  namespace: databases
spec:
  instances: 1
  storage:
    size: 10Gi
  bootstrap:
    initdb:
      database: alerthub
      owner: alerthub_user
```

## Requirements

Deploy infrastructure resources using CRDs:

1. Create namespaces: databases, kafka, messaging, storage

2. PostgreSQL (CloudNative-PG):

3. Redis/Valkey, Kafka (Strimzi), MongoDB (Percona), RabbitMQ, and SeaweedFS CRDs as specified in PRD

4. Create ConfigMaps and Secrets for connection strings
5. Apply Network Policies for service isolation

## Acceptance Criteria

1. Verify all pods reach Running state
2. Test connectivity to each service from a debug pod
3. Validate PostgreSQL accepts connections
4. Confirm Redis responds to PING
5. Verify Kafka topic creation works
6. Test MongoDB connection

## Constraints

- Match existing codebase patterns and style
- Create PR with atomic, well-described commits
- Include unit tests for new functionality
- PR title: `feat(task-1): Setup Infrastructure (Bolt - Kubernetes)`

## Resources

- PRD: `.tasks/docs/prd.txt`
