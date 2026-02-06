# Subtask 1.2: Deploy MongoDB Cluster

## Parent Task
Task 1

## Agent
mongo-deployer

## Parallelizable
Yes

## Description
Deploy and configure Percona MongoDB operator and cluster with replica sets for high availability.

## Details
- Install Percona MongoDB Operator
- Create MongoDB cluster with replica set topology
- Configure storage with appropriate IOPS
- Set up monitoring with custom metrics
- Implement backup strategy to object storage
- Configure network policies for database isolation

## Deliverables
- `mongodb-operator.yaml` - Operator deployment
- `mongodb-cluster.yaml` - MongoDB cluster CR
- `mongodb-backup.yaml` - Backup configuration
- `mongodb-secrets.yaml` - Credentials

## Acceptance Criteria
- [ ] MongoDB operator is Running
- [ ] MongoDB cluster pods are Running
- [ ] Primary and secondary pods are reachable
- [ ] CRUD operations work correctly
- [ ] Backup/restore procedure verified

## Testing Strategy
- Connect with mongosh and verify replica set status
- Insert documents and verify replication
- Test failover by deleting primary pod
