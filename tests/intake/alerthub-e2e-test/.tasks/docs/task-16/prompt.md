# Task 16: Deploy AlertHub Infrastructure (Bolt - Kubernetes)

**Agent**: bolt | **Language**: yaml

## Role

You are a Senior DevOps Engineer with expertise in Kubernetes, GitOps, and CI/CD implementing Task 16.

## Goal

Provision all required infrastructure components including PostgreSQL, Redis/Valkey, Kafka, MongoDB, RabbitMQ, and SeaweedFS using Kubernetes operators. Configure networking, secrets, and persistent storage for the multi-service architecture.

## Requirements

1. Deploy PostgreSQL cluster using CloudNative-PG operator:
   - Create namespace 'databases'
   - Apply Cluster CRD with 10Gi storage
   - Initialize 'alerthub' database with 'alerthub_user'
   - Create secret with connection string: postgresql://alerthub_user:password@alerthub-postgres-rw.databases.svc.cluster.local:5432/alerthub

2. Deploy Valkey (Redis) using Redis Operator:
   - Apply Redis CRD in 'databases' namespace
   - Configure 2Gi memory limit
   - Expose ClusterIP service on port 6379
   - Create ConfigMap with connection: alerthub-valkey.databases.svc.cluster.local:6379

3. Deploy Kafka using Strimzi operator:
   - Create namespace 'kafka'
   - Apply Kafka CRD with single broker
   - Configure internal listener on port 9092
   - Create topics: notifications-events, delivery-events, dlq
   - Bootstrap servers: alerthub-kafka-kafka-bootstrap.kafka.svc.cluster.local:9092

4. Deploy MongoDB using Percona operator:
   - Apply PerconaServerMongoDB CRD in 'databases' namespace
   - Configure replica set 'rs0' with single instance
   - Create database 'alerthub_integrations'
   - Connection string: mongodb://alerthub-mongodb-rs0-0.alerthub-mongodb-rs0.databases.svc.cluster.local:27017/alerthub_integrations

5. Deploy RabbitMQ cluster:
   - Create namespace 'messaging'
   - Apply RabbitmqCluster CRD
   - Create queues: delivery-tasks, webhook-tasks
   - Connection: amqp://alerthub-rabbitmq.messaging.svc.cluster.local:5672

6. Deploy SeaweedFS for object storage:
   - Deploy master (1 replica) and volume servers (2 replicas)
   - Configure S3-compatible API endpoint
   - Create bucket 'alerthub-attachments'
   - Endpoint: http://seaweedfs-s3.storage.svc.cluster.local:8333

7. Create shared ConfigMap 'alerthub-config' with all connection strings
8. Create Secrets for database credentials, API keys
9. Apply NetworkPolicies for service isolation
10. Configure PersistentVolumeClaims for stateful workloads

## Acceptance Criteria

1. Verify all operators are installed and running
2. Check pod status for all infrastructure components (kubectl get pods -A)
3. Test PostgreSQL connectivity: psql -h alerthub-postgres-rw.databases.svc.cluster.local -U alerthub_user -d alerthub
4. Test Redis: redis-cli -h alerthub-valkey.databases.svc.cluster.local ping
5. Test Kafka: kafka-console-producer --bootstrap-server alerthub-kafka-kafka-bootstrap.kafka.svc.cluster.local:9092 --topic test
6. Test MongoDB: mongosh mongodb://alerthub-mongodb-rs0-0.alerthub-mongodb-rs0.databases.svc.cluster.local:27017
7. Test RabbitMQ management UI and queue creation
8. Test SeaweedFS S3 API with curl
9. Verify ConfigMap and Secrets are created and accessible
10. Run connectivity tests from a test pod to ensure network policies allow required traffic

## Constraints

- Match existing codebase patterns and style
- Create PR with atomic, well-described commits
- Include unit tests for new functionality
- PR title: `feat(task-16): Deploy AlertHub Infrastructure (Bolt - Kubernetes)`

## Resources

- PRD: `.tasks/docs/prd.txt`
