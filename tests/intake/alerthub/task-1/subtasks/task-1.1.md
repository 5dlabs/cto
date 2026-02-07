# Subtask 1.1: Deploy PostgreSQL Cluster

## Parent Task
Task 1

## Agent
postgres-deployer

## Parallelizable
Yes

## Description
Deploy and configure CloudNative-PG PostgreSQL cluster on Kubernetes with high availability, connection pooling, and backup configuration.

## Details
- Create PostgreSQLCluster CR with appropriate resource limits
- Configure connection pooling with PgBouncer sidecar
- Set up WAL archiving to object storage
- Configure replica reads for read scaling
- Implement backup schedule with pgBackRest
- Create dedicated namespace for database workloads

## Deliverables
- `postgresql-cluster.yaml` - CloudNative-PG Cluster CR
- `pgbouncer-configmap.yaml` - Connection pooling config
- `backup-schedule.yaml` - pgBackRest schedule
- `postgresql-secrets.yaml` - Database credentials

## Acceptance Criteria
- [ ] PostgreSQL cluster pods are Running (primary + replicas)
- [ ] Database accepts connections on port 5432
- [ ] PVC is bound with persistent storage
- [ ] Replication is working (data synced to replicas)
- [ ] Backup completes successfully
- [ ] Connection pool accepts connections

## Testing Strategy
- Connect with psql and verify replication status
- Create test database and tables
- Verify backup/restore procedure
- Test connection pooling throughput
