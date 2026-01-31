# PostgreSQL with CloudNative-PG

This directory contains Kubernetes manifests for deploying a highly available PostgreSQL cluster using the CloudNative-PG operator.

## Overview

- **Operator**: CloudNative-PG (Cloud Native PostgreSQL)
- **PostgreSQL Version**: 16
- **High Availability**: 3-instance cluster (1 primary + 2 read replicas)
- **Storage**: 10Gi per instance + 5Gi WAL storage
- **Connection Pooling**: PgBouncer (separate RW and RO poolers)
- **Backup**: Point-in-time recovery with scheduled backups

## Architecture

### Cluster Configuration

The PostgreSQL cluster consists of:
- **Primary Instance**: Handles all write operations
- **2 Read Replicas**: Synchronous replication for high availability
- **Automatic Failover**: CloudNative-PG handles automatic failover
- **Pod Anti-Affinity**: Instances spread across different nodes

### Connection Pooling

Two PgBouncer poolers are configured:
- **RW Pooler** (`postgres-cluster-pooler-rw`): Routes to primary for read-write operations
- **RO Pooler** (`postgres-cluster-pooler-ro`): Routes to replicas for read-only operations

Each pooler runs 3 instances for high availability.

### Backup Strategy

- **Continuous Archiving**: WAL files continuously archived to S3-compatible storage
- **Scheduled Backups**:
  - Daily full backup at 2 AM
  - Hourly incremental backups
- **Retention**: 30 days
- **Point-in-Time Recovery**: Enabled for disaster recovery

## Prerequisites

1. **CloudNative-PG Operator** must be installed in the cluster:
   ```bash
   kubectl apply -f https://raw.githubusercontent.com/cloudnative-pg/cloudnative-pg/release-1.22/releases/cnpg-1.22.0.yaml
   ```

2. **Storage Class**: A StorageClass named `standard` must be available
   - For production, use a storage class with good I/O performance

3. **Backup Storage**: S3-compatible storage (MinIO, AWS S3, etc.)
   - Update backup credentials in `cluster.yaml`
   - Update endpoint URL to match your storage service

4. **Certificate Management** (optional but recommended):
   - TLS certificates can be provided via cert-manager
   - If not provided, operator generates self-signed certificates

## Deployment

### 1. Update Secrets

Before deploying, update the following secrets in `cluster.yaml`:

```yaml
# Application user password
postgres-cluster-app-user:
  password: <strong-password>

# Superuser password
postgres-cluster-superuser:
  password: <strong-password>

# Backup storage credentials
backup-storage-credentials:
  ACCESS_KEY_ID: <your-access-key>
  ACCESS_SECRET_KEY: <your-secret-key>
```

Also update the pooler auth query secret in `pooler.yaml`:

```yaml
postgres-cluster-pooler-auth-query:
  password: <same-as-superuser-password>
```

### 2. Deploy with Kustomize

```bash
# Deploy all resources
kubectl apply -k /workspace/infra/postgresql/

# Or deploy using kustomize build
kustomize build /workspace/infra/postgresql/ | kubectl apply -f -
```

### 3. Verify Deployment

```bash
# Check cluster status
kubectl get cluster -n databases

# Check pods
kubectl get pods -n databases -l cnpg.io/cluster=postgres-cluster

# Check poolers
kubectl get pooler -n databases

# Check services
kubectl get svc -n databases

# View cluster details
kubectl describe cluster postgres-cluster -n databases
```

## Connecting to PostgreSQL

### Via Read-Write Pooler (Primary)

For applications that need to write data:

```bash
# Service endpoint
postgres-cluster-pooler-rw.databases.svc.cluster.local:5432

# Connection string
postgresql://app:<password>@postgres-cluster-pooler-rw.databases.svc.cluster.local:5432/app
```

### Via Read-Only Pooler (Replicas)

For applications that only read data:

```bash
# Service endpoint
postgres-cluster-pooler-ro.databases.svc.cluster.local:5432

# Connection string
postgresql://app:<password>@postgres-cluster-pooler-ro.databases.svc.cluster.local:5432/app
```

### Direct Connection (without pooler)

For administrative tasks or connection-intensive operations:

```bash
# Primary (RW)
postgres-cluster-rw.databases.svc.cluster.local:5432

# Replicas (RO)
postgres-cluster-ro.databases.svc.cluster.local:5432

# Any instance (for admin)
postgres-cluster-r.databases.svc.cluster.local:5432
```

## Operations

### Scaling

To change the number of instances:

```bash
kubectl cnpg scale postgres-cluster -n databases --replicas 5
```

Or edit `cluster.yaml` and reapply.

### Manual Backup

Trigger an immediate backup:

```bash
kubectl cnpg backup postgres-cluster -n databases
```

### Point-in-Time Recovery

To restore to a specific point in time, create a new cluster with bootstrap configuration:

```yaml
bootstrap:
  recovery:
    source: postgres-cluster
    recoveryTarget:
      targetTime: "2024-01-31 12:00:00"
```

### Failover

Automatic failover is handled by the operator. To manually promote a replica:

```bash
kubectl cnpg promote postgres-cluster 2 -n databases
```

### Monitoring

The cluster exposes Prometheus metrics. If you have Prometheus Operator installed, PodMonitors are automatically created.

View metrics:
```bash
# Port-forward to a PostgreSQL pod
kubectl port-forward -n databases postgres-cluster-1 9187:9187

# Access metrics
curl http://localhost:9187/metrics
```

### Logs

View PostgreSQL logs:

```bash
# Instance logs
kubectl logs -n databases postgres-cluster-1 -f

# Pooler logs
kubectl logs -n databases -l cnpg.io/poolerName=postgres-cluster-pooler-rw -f
```

### Certificate Rotation

Certificates are automatically managed by the operator. To manually trigger rotation:

```bash
kubectl cnpg certificate postgres-cluster -n databases
```

## Maintenance

### Upgrading PostgreSQL

To upgrade to a new major version:

1. Update the `imageName` in `cluster.yaml`
2. Apply the changes
3. The operator performs a rolling update

For major version upgrades, consider using `pg_upgrade` or logical replication.

### Updating Configuration

1. Edit `cluster.yaml` with new PostgreSQL parameters
2. Apply changes:
   ```bash
   kubectl apply -k /workspace/infra/postgresql/
   ```
3. The operator performs a rolling restart if needed

### Backup Verification

List available backups:

```bash
kubectl get backup -n databases
```

Check backup status:

```bash
kubectl describe backup <backup-name> -n databases
```

## Troubleshooting

### Cluster Not Starting

Check operator logs:
```bash
kubectl logs -n cnpg-system deployment/cnpg-controller-manager
```

Check cluster status:
```bash
kubectl describe cluster postgres-cluster -n databases
```

### Replication Issues

Check replication status:
```bash
kubectl exec -n databases postgres-cluster-1 -- psql -U postgres -c "SELECT * FROM pg_stat_replication;"
```

### Storage Issues

Check PVC status:
```bash
kubectl get pvc -n databases
```

### Connection Issues

Test connectivity from within the cluster:
```bash
kubectl run -it --rm --image=postgres:16 psql-client -n databases -- \
  psql postgresql://app:<password>@postgres-cluster-pooler-rw:5432/app
```

## Security Considerations

### Production Checklist

- [ ] Change all default passwords
- [ ] Use external secret management (e.g., Vault, External Secrets Operator)
- [ ] Enable TLS with proper certificates (use cert-manager)
- [ ] Configure network policies to restrict access
- [ ] Enable encryption at rest for storage
- [ ] Secure backup storage with encryption and access controls
- [ ] Review and harden PostgreSQL parameters
- [ ] Set up proper RBAC for cluster access
- [ ] Enable audit logging
- [ ] Implement backup verification and testing

### Network Policies

Example network policy to restrict access:

```yaml
apiVersion: networking.k8s.io/v1
kind: NetworkPolicy
metadata:
  name: postgres-cluster-netpol
  namespace: databases
spec:
  podSelector:
    matchLabels:
      cnpg.io/cluster: postgres-cluster
  policyTypes:
    - Ingress
  ingress:
    - from:
        - namespaceSelector:
            matchLabels:
              name: application-namespace
      ports:
        - protocol: TCP
          port: 5432
```

## Resource Requirements

### Minimum Resources (per instance)

- **CPU**: 500m request, 1000m limit
- **Memory**: 512Mi request, 1Gi limit
- **Storage**: 10Gi data + 5Gi WAL

### Recommended Production Resources

- **CPU**: 2000m request, 4000m limit
- **Memory**: 4Gi request, 8Gi limit
- **Storage**: 100Gi+ data + 20Gi WAL (adjust based on workload)

Update resource requests/limits in `cluster.yaml` based on your workload.

## References

- [CloudNative-PG Documentation](https://cloudnative-pg.io/)
- [PostgreSQL Documentation](https://www.postgresql.org/docs/16/)
- [PgBouncer Documentation](https://www.pgbouncer.org/)
- [CloudNative-PG Examples](https://github.com/cloudnative-pg/cloudnative-pg/tree/main/docs/src/samples)

## Support

For issues specific to CloudNative-PG operator:
- GitHub: https://github.com/cloudnative-pg/cloudnative-pg
- Slack: #cloudnative-pg on Kubernetes Slack
