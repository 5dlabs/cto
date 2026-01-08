# Task 19: Setup AlertHub Infrastructure (Bolt - Kubernetes)

**Agent**: bolt | **Language**: yaml

## Role

You are a Senior DevOps Engineer with expertise in Kubernetes, GitOps, and CI/CD implementing Task 19.

## Goal

Provision all infrastructure resources required by AlertHub: PostgreSQL for relational data, Redis/Valkey for caching and rate limiting, Kafka for event streaming, MongoDB for flexible document storage, RabbitMQ for task queues, and SeaweedFS for object storage. This foundational task must complete before any backend services can be deployed.

## Requirements

1. Deploy PostgreSQL cluster using CloudNative-PG operator:
   - Create Cluster CR with 1 instance, 20Gi storage on mayastor
   - Initialize 'alerthub' database with 'alerthub_user' owner
   - Configure max_connections=200, shared_buffers=256MB
   - Create tables: users, tenants, notifications, rules, audit_logs
   - Connection string: postgresql://alerthub_user:<password>@alerthub-postgres-rw.databases.svc:5432/alerthub

2. Deploy Redis/Valkey using Redis Operator:
   - Create Redis CR with valkey:7.2-alpine image
   - Allocate 5Gi persistent storage on mayastor
   - Configure resource limits: 500m CPU, 1Gi memory
   - Connection string: redis://alerthub-valkey.databases.svc:6379

3. Deploy Kafka cluster using Strimzi operator:
   - Create Kafka CR with version 3.8.0, 1 replica
   - Configure topics: alerthub.notifications.created (6 partitions), alerthub.notifications.delivered (3 partitions), alerthub.notifications.failed (3 partitions)
   - Set retention: 7 days for created/delivered, 14 days for failed
   - Bootstrap servers: alerthub-kafka-kafka-bootstrap.kafka.svc:9092

4. Deploy MongoDB using Percona operator:
   - Create PerconaServerMongoDB CR with version 7.0.14-8
   - Configure rs0 replica set with 1 member, 10Gi storage
   - Create collections: integrations, templates, delivery_logs
   - Connection string: mongodb://alerthub-mongodb-rs0.databases.svc:27017

5. Deploy RabbitMQ using RabbitMQ operator:
   - Create RabbitmqCluster CR with 1 replica, 10Gi storage
   - Configure queues with DLQ: integration.slack.delivery, integration.discord.delivery, integration.email.delivery, integration.webhook.delivery
   - Connection string: amqp://alerthub-rabbitmq.messaging.svc:5672

6. Configure SeaweedFS for object storage:
   - Create S3 buckets: alerthub-attachments, alerthub-exports, alerthub-media
   - S3 endpoint: http://seaweedfs-filer.seaweedfs.svc:8333

7. Create shared ConfigMap 'alerthub-config' with all connection strings

8. Create Secrets for database credentials, API keys

## Acceptance Criteria

1. Verify PostgreSQL cluster status: kubectl get cluster alerthub-postgres -n databases (should show 1/1 ready)
2. Test PostgreSQL connection: psql -h alerthub-postgres-rw.databases.svc -U alerthub_user -d alerthub -c 'SELECT 1'
3. Verify Redis connectivity: redis-cli -h alerthub-valkey.databases.svc ping (should return PONG)
4. Check Kafka topics: kubectl exec -it alerthub-kafka-kafka-0 -n kafka -- bin/kafka-topics.sh --list --bootstrap-server localhost:9092
5. Test MongoDB connection: mongosh mongodb://alerthub-mongodb-rs0.databases.svc:27017 --eval 'db.runCommand({ping:1})'
6. Verify RabbitMQ queues: kubectl exec -it alerthub-rabbitmq-server-0 -n messaging -- rabbitmqctl list_queues
7. Test SeaweedFS S3 API: aws s3 ls s3://alerthub-attachments --endpoint-url http://seaweedfs-filer.seaweedfs.svc:8333
8. Validate all resources have PersistentVolumeClaims bound
9. Check operator logs for any errors during provisioning

## Constraints

- Match existing codebase patterns and style
- Create PR with atomic, well-described commits
- Include unit tests for new functionality
- PR title: `feat(task-19): Setup AlertHub Infrastructure (Bolt - Kubernetes)`

## Resources

- PRD: `.tasks/docs/prd.txt`
