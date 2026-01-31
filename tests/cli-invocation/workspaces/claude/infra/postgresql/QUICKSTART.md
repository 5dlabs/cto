# PostgreSQL Quick Start Guide

## Prerequisites

Install CloudNative-PG operator:

```bash
kubectl apply -f https://raw.githubusercontent.com/cloudnative-pg/cloudnative-pg/release-1.22/releases/cnpg-1.22.0.yaml
```

## Quick Deploy

### 1. Update Secrets (REQUIRED)

Edit `cluster.yaml` and `pooler.yaml` to set secure passwords:

- `postgres-cluster-app-user` password
- `postgres-cluster-superuser` password
- `backup-storage-credentials` (ACCESS_KEY_ID and ACCESS_SECRET_KEY)
- `postgres-cluster-pooler-auth-query` password (must match superuser password)

### 2. Update Backup Configuration

Edit the backup configuration in `cluster.yaml`:

```yaml
backup:
  barmanObjectStore:
    destinationPath: s3://your-bucket/postgres-cluster  # Update this
    endpointURL: http://your-minio-endpoint:9000        # Update this
```

If you don't have S3/MinIO ready, you can comment out the entire `backup` section for testing.

### 3. Deploy

```bash
kubectl apply -k /workspace/infra/postgresql/
```

### 4. Verify

```bash
# Check cluster status
kubectl get cluster -n databases

# Wait for cluster to be ready
kubectl wait --for=condition=Ready cluster/postgres-cluster -n databases --timeout=300s

# Check pods
kubectl get pods -n databases

# Check services
kubectl get svc -n databases
```

## Connect to Database

### From within Kubernetes

```bash
# Read-Write (Primary)
kubectl run -it --rm psql-client -n databases --image=postgres:16 -- \
  psql postgresql://app:PASSWORD@postgres-cluster-pooler-rw:5432/app

# Read-Only (Replicas)
kubectl run -it --rm psql-client -n databases --image=postgres:16 -- \
  psql postgresql://app:PASSWORD@postgres-cluster-pooler-ro:5432/app
```

Replace `PASSWORD` with your app user password.

### Connection Strings for Applications

```bash
# Write operations
RW_DSN="postgresql://app:PASSWORD@postgres-cluster-pooler-rw.databases.svc.cluster.local:5432/app"

# Read operations
RO_DSN="postgresql://app:PASSWORD@postgres-cluster-pooler-ro.databases.svc.cluster.local:5432/app"
```

## Common Commands

```bash
# View cluster status
kubectl cnpg status postgres-cluster -n databases

# View logs
kubectl logs -n databases -l cnpg.io/cluster=postgres-cluster --tail=100 -f

# Trigger manual backup
kubectl cnpg backup postgres-cluster -n databases

# List backups
kubectl get backup -n databases

# Scale cluster
kubectl cnpg scale postgres-cluster -n databases --replicas 5

# Promote standby to primary (manual failover)
kubectl cnpg promote postgres-cluster 2 -n databases

# View replication status
kubectl exec -n databases postgres-cluster-1 -- \
  psql -U postgres -c "SELECT * FROM pg_stat_replication;"

# Check database size
kubectl exec -n databases postgres-cluster-1 -- \
  psql -U postgres -c "SELECT pg_size_pretty(pg_database_size('app'));"
```

## Testing High Availability

### Simulate pod failure

```bash
# Delete primary pod (automatic failover will occur)
kubectl delete pod -n databases postgres-cluster-1

# Watch failover process
kubectl get pods -n databases -w

# Check new primary
kubectl cnpg status postgres-cluster -n databases
```

## Cleanup

```bash
# Delete all resources
kubectl delete -k /workspace/infra/postgresql/

# Delete PVCs (WARNING: This deletes all data)
kubectl delete pvc -n databases -l cnpg.io/cluster=postgres-cluster
```

## Troubleshooting

### Cluster stuck in "Creating" state

```bash
# Check operator logs
kubectl logs -n cnpg-system deployment/cnpg-controller-manager

# Check cluster events
kubectl describe cluster postgres-cluster -n databases

# Check pod events
kubectl describe pod -n databases postgres-cluster-1
```

### Connection refused

```bash
# Check if pods are running
kubectl get pods -n databases

# Check service endpoints
kubectl get endpoints -n databases

# Check pooler status
kubectl get pooler -n databases
kubectl logs -n databases -l cnpg.io/poolerName=postgres-cluster-pooler-rw
```

### Backup failures

```bash
# Check backup status
kubectl get backup -n databases
kubectl describe backup <backup-name> -n databases

# Check storage credentials
kubectl get secret backup-storage-credentials -n databases -o yaml

# Test S3 connectivity
kubectl run -it --rm aws-cli -n databases --image=amazon/aws-cli -- \
  s3 ls --endpoint-url=http://your-endpoint:9000
```

## Next Steps

- Review and adjust resource requests/limits in `cluster.yaml`
- Set up monitoring with Prometheus and Grafana
- Configure network policies for security
- Set up automated backup verification
- Review PostgreSQL parameters for your workload
- Implement connection pooling best practices
- Set up database maintenance jobs (VACUUM, ANALYZE)

For detailed documentation, see [README.md](README.md).
