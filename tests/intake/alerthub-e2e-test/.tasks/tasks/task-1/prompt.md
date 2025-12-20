# Task 1: Infrastructure Setup with Kubernetes Operators

## Agent: Bolt
## Priority: High

## Objective
Set up the AlertHub infrastructure using Kubernetes operators for databases, caches, and message queues.

## Required Infrastructure

### 1. PostgreSQL (CloudNative-PG)
Create a PostgreSQL cluster for:
- Notification persistence
- Tenant data
- User data
- Audit logs

```yaml
apiVersion: postgresql.cnpg.io/v1
kind: Cluster
metadata:
  name: alerthub-postgres
  namespace: alerthub
spec:
  instances: 2
  storage:
    size: 10Gi
    storageClass: mayastor
```

### 2. Redis/Valkey (Redis Operator)
Set up Redis for:
- Rate limiting
- Deduplication cache
- Session cache
- Analytics aggregation

### 3. Kafka (Strimzi)
Configure Kafka for:
- Event streaming between services
- Notification events
- Integration events

### 4. MongoDB (Percona)
Deploy MongoDB for:
- Integration configs
- Message templates

### 5. RabbitMQ (RabbitMQ Operator)
Set up RabbitMQ for:
- Task queue for delivery jobs

## Output
- Create ConfigMap `alerthub-infra-config` with connection strings
- All resources deployed to `alerthub` namespace
- Health checks passing for all databases

## Acceptance Criteria
- [ ] PostgreSQL cluster is running with 2 replicas
- [ ] Redis/Valkey is accessible
- [ ] Kafka brokers are ready
- [ ] MongoDB is accepting connections
- [ ] RabbitMQ is available with management UI
- [ ] ConfigMap contains all connection strings

