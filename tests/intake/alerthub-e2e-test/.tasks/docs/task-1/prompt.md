# Task 1: Setup Infrastructure Resources (Bolt - Kubernetes)

**Agent**: bolt | **Language**: yaml

## Role

You are a Senior DevOps Engineer with expertise in Kubernetes, GitOps, and CI/CD implementing Task 1.

## Goal

Provision all infrastructure components required by AlertHub services including databases, caches, message queues, and storage using Kubernetes operators

## Requirements

1. Deploy PostgreSQL cluster using CloudNative-PG operator:
   - Create Cluster CR with 1 instance, 20Gi storage
   - Initialize 'alerthub' database with 'alerthub_user' owner
   - Configure connection pooling (max_connections: 200)
   - Set resource limits (512Mi-2Gi memory, 250m-1000m CPU)

2. Deploy Redis/Valkey using Redis Operator:
   - Create Redis CR with valkey:7.2-alpine image
   - Configure 5Gi persistent storage with mayastor
   - Set resource limits (256Mi-1Gi memory, 100m-500m CPU)

3. Deploy Kafka using Strimzi operator:
   - Create Kafka CR with version 3.8.0, 1 replica
   - Configure internal listener on port 9092
   - Create topics: alerthub.notifications.created (6 partitions), alerthub.notifications.delivered (3 partitions), alerthub.notifications.failed (3 partitions)
   - Set retention to 7 days for created/delivered, 14 days for failed
   - Configure 20Gi persistent storage

4. Deploy MongoDB using Percona operator:
   - Create PerconaServerMongoDB CR with version 7.0.14-8
   - Configure replica set 'rs0' with 1 member
   - Set 10Gi persistent storage with mayastor

5. Deploy RabbitMQ using RabbitMQ operator:
   - Create RabbitmqCluster CR with 1 replica
   - Configure 10Gi persistent storage
   - Set resource limits (512Mi-1Gi memory, 200m-500m CPU)

6. Setup SeaweedFS for S3-compatible storage:
   - Deploy SeaweedFS filer service
   - Create buckets: alerthub-attachments, alerthub-exports, alerthub-media

7. Create ConfigMaps with connection strings:
   - postgres-config: PostgreSQL connection URL
   - redis-config: Redis connection URL
   - kafka-config: Kafka bootstrap servers
   - mongodb-config: MongoDB connection URL
   - rabbitmq-config: RabbitMQ connection URL
   - seaweedfs-config: S3 endpoint URL

8. Create Secrets for credentials:
   - postgres-secret: database password
   - mongodb-secret: database password
   - rabbitmq-secret: admin credentials

## Acceptance Criteria

1. Verify all operator CRs are in Ready state
2. Test connectivity to each service from a debug pod
3. Verify Kafka topics exist with correct partition counts
4. Confirm SeaweedFS buckets are created
5. Validate ConfigMaps and Secrets contain correct values
6. Run integration tests connecting to each service

## Constraints

- Match existing codebase patterns and style
- Create PR with atomic, well-described commits
- Include unit tests for new functionality
- PR title: `feat(task-1): Setup Infrastructure Resources (Bolt - Kubernetes)`

## Resources

- PRD: `.tasks/docs/prd.txt`
