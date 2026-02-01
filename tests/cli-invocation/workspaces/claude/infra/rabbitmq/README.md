# RabbitMQ Cluster - Production Configuration

This directory contains Kubernetes manifests for deploying a highly available RabbitMQ cluster using the RabbitMQ Cluster Operator.

## Overview

- **Cluster Name**: rabbitmq-cluster
- **Namespace**: messaging
- **RabbitMQ Version**: 3.12-management
- **Replicas**: 3 nodes (HA configuration)
- **Storage**: 10Gi per node with persistent volumes
- **Resource Requests**: 500m CPU, 1Gi Memory per node
- **Resource Limits**: 2000m CPU, 2Gi Memory per node

## Architecture

### High Availability Configuration

- **3-node cluster** with automatic leader election
- **Pod anti-affinity** rules to distribute pods across nodes
- **PodDisruptionBudget** ensures minimum 2 pods available during disruptions
- **Automatic cluster formation** using Kubernetes peer discovery
- **Partition handling**: autoheal mode for network split recovery

### Storage

- Persistent volumes with 10Gi storage per node
- Uses default StorageClass (can be customized in cluster.yaml)
- Data persists across pod restarts and rescheduling

### Networking

- **AMQP Service** (rabbitmq-cluster-amqp): Port 5672 (plain), 5671 (TLS)
- **Management Service** (rabbitmq-cluster-management): Port 15672 (HTTP UI/API)
- **Metrics Service** (rabbitmq-cluster-metrics): Port 15692 (Prometheus)
- **Headless Service** (rabbitmq-cluster-nodes): For StatefulSet DNS and inter-node communication

### Security

- **RBAC**: ServiceAccount with minimal permissions for peer discovery
- **NetworkPolicy**: Restricts ingress/egress to necessary ports and namespaces
- **Pod Security**: Non-root user (UID 999), dropped capabilities
- **Secrets**: User credentials stored in Kubernetes secrets

## Pre-requisites

1. **RabbitMQ Cluster Operator** must be installed:
   ```bash
   kubectl apply -f "https://github.com/rabbitmq/cluster-operator/releases/latest/download/cluster-operator.yml"
   ```

2. **StorageClass** configured for persistent volumes

3. **Prometheus Operator** (optional, for monitoring resources)

## Components

### Core Resources

- **namespace.yaml**: Creates the `messaging` namespace
- **cluster.yaml**: Main RabbitMQ cluster configuration
- **definitions.yaml**: User, vhost, and policy definitions
- **service.yaml**: Service definitions for AMQP, Management, and Metrics
- **rbac.yaml**: ServiceAccount and RBAC for peer discovery
- **configmap.yaml**: Additional configuration and initialization scripts

### High Availability & Reliability

- **poddisruptionbudget.yaml**: Ensures minimum availability during maintenance
- **networkpolicy.yaml**: Network isolation and security policies

### Monitoring

- **monitoring.yaml**: ServiceMonitor and PrometheusRule for Prometheus Operator
  - Metrics collection every 30 seconds
  - Comprehensive alerting rules for cluster health, memory, disk, queues, and connections

## Configuration Details

### Users and Credentials

Default users (defined in definitions.yaml - **CHANGE THESE IN PRODUCTION**):

1. **admin** - Administrator access to all vhosts
   - Password: `changeme123`
   - Tags: `administrator`

2. **app-user** - Application user with management access
   - Password: `changeme456`
   - Tags: `management`
   - Access to: /production, /staging, /development vhosts

3. **monitoring** - Read-only monitoring user
   - Password: `changeme789`
   - Tags: `monitoring`

### Virtual Hosts

- `/` - Default vhost
- `/production` - Production environment (HA policy: replicate to all nodes)
- `/staging` - Staging environment (HA policy: replicate to 2 nodes)
- `/development` - Development environment

### Policies

- **Production vhost**: All queues replicated to all nodes (ha-all)
- **Staging vhost**: All queues replicated to exactly 2 nodes
- Automatic synchronization enabled

### Enabled Plugins

- rabbitmq_management - Web UI and HTTP API
- rabbitmq_prometheus - Prometheus metrics
- rabbitmq_shovel - Message forwarding
- rabbitmq_shovel_management - Shovel management UI
- rabbitmq_federation - Federated exchanges and queues
- rabbitmq_federation_management - Federation management UI
- rabbitmq_consistent_hash_exchange - Consistent hash exchange type

## Deployment

### Using kubectl

```bash
# Deploy all resources
kubectl apply -k /workspace/infra/rabbitmq/

# Or deploy individually
kubectl apply -f namespace.yaml
kubectl apply -f rbac.yaml
kubectl apply -f configmap.yaml
kubectl apply -f definitions.yaml
kubectl apply -f cluster.yaml
kubectl apply -f service.yaml
kubectl apply -f networkpolicy.yaml
kubectl apply -f poddisruptionbudget.yaml
kubectl apply -f monitoring.yaml
```

### Using Kustomize

```bash
# Preview generated manifests
kubectl kustomize /workspace/infra/rabbitmq/

# Deploy
kubectl apply -k /workspace/infra/rabbitmq/
```

## Verification

### Check Cluster Status

```bash
# Check if pods are running
kubectl get pods -n messaging

# Check cluster status
kubectl exec -n messaging rabbitmq-cluster-server-0 -- rabbitmq-diagnostics cluster_status

# Check node health
kubectl exec -n messaging rabbitmq-cluster-server-0 -- rabbitmq-diagnostics status
```

### Access Management UI

```bash
# Port forward to access locally
kubectl port-forward -n messaging svc/rabbitmq-cluster-management 15672:15672

# Access at http://localhost:15672
# Default credentials: admin / changeme123
```

### Test AMQP Connection

```bash
# Port forward AMQP port
kubectl port-forward -n messaging svc/rabbitmq-cluster-amqp 5672:5672

# Use amqp://admin:changeme123@localhost:5672/
```

## Monitoring

### Prometheus Metrics

Metrics are exposed on port 15692 at `/metrics`:

```bash
kubectl port-forward -n messaging svc/rabbitmq-cluster-metrics 15692:15692
curl http://localhost:15692/metrics
```

### Key Metrics

- `rabbitmq_queue_messages` - Messages in queues
- `rabbitmq_queue_consumers` - Active consumers
- `rabbitmq_connections` - Active connections
- `rabbitmq_channels` - Active channels
- `rabbitmq_process_resident_memory_bytes` - Memory usage
- `rabbitmq_disk_space_available_bytes` - Disk space

### Alerts

PrometheusRule includes alerts for:
- Cluster availability (nodes down)
- Memory pressure (>90% and >95%)
- Disk space low (<20% and <10%)
- File descriptors high (>80%)
- Queue message buildup
- No consumers on queues with messages
- Too many connections

## Operations

### Scaling

```bash
# Scale to 5 nodes
kubectl patch rabbitmqcluster rabbitmq-cluster -n messaging --type='merge' -p '{"spec":{"replicas":5}}'

# Verify scaling
kubectl get pods -n messaging -l app.kubernetes.io/name=rabbitmq-cluster
```

### Backup

```bash
# Backup definitions
kubectl exec -n messaging rabbitmq-cluster-server-0 -- rabbitmqctl export_definitions /tmp/definitions.json
kubectl cp messaging/rabbitmq-cluster-server-0:/tmp/definitions.json ./definitions-backup.json

# Backup persistent volumes (use your backup solution)
```

### Restore

```bash
# Restore definitions
kubectl cp ./definitions-backup.json messaging/rabbitmq-cluster-server-0:/tmp/definitions.json
kubectl exec -n messaging rabbitmq-cluster-server-0 -- rabbitmqctl import_definitions /tmp/definitions.json
```

### Upgrade

```bash
# Update image in cluster.yaml, then apply
kubectl apply -f cluster.yaml

# Operator will perform rolling update
kubectl rollout status statefulset/rabbitmq-cluster-server -n messaging
```

## Troubleshooting

### Check Logs

```bash
# View pod logs
kubectl logs -n messaging rabbitmq-cluster-server-0 -f

# View operator logs
kubectl logs -n rabbitmq-system deployment/rabbitmq-cluster-operator -f
```

### Common Issues

1. **Pods not starting**: Check PVC status and StorageClass availability
   ```bash
   kubectl get pvc -n messaging
   kubectl describe pvc -n messaging
   ```

2. **Cluster formation issues**: Check peer discovery permissions
   ```bash
   kubectl get role,rolebinding -n messaging
   kubectl logs -n messaging rabbitmq-cluster-server-0 | grep -i "peer discovery"
   ```

3. **Memory pressure**: Increase resource limits or add more nodes
   ```bash
   kubectl top pods -n messaging
   ```

4. **Connection failures**: Check NetworkPolicy and Services
   ```bash
   kubectl get networkpolicy,svc -n messaging
   ```

## Security Considerations

### Change Default Credentials

Before deploying to production, update the following in `definitions.yaml`:

```yaml
stringData:
  definitions.json: |
    {
      "users": [
        {
          "name": "admin",
          "password": "<strong-password>",  # Change this!
          ...
        }
      ]
    }
```

### Enable TLS

Uncomment and configure TLS section in `cluster.yaml`:

```yaml
tls:
  secretName: rabbitmq-tls-secret
  caSecretName: rabbitmq-ca-secret
  disableNonTLSListeners: true
```

Create TLS secrets:

```bash
kubectl create secret tls rabbitmq-tls-secret \
  --cert=path/to/tls.crt \
  --key=path/to/tls.key \
  -n messaging

kubectl create secret generic rabbitmq-ca-secret \
  --from-file=ca.crt=path/to/ca.crt \
  -n messaging
```

### Network Isolation

Adjust NetworkPolicy in `networkpolicy.yaml` to match your environment:

```yaml
ingress:
  - from:
      - namespaceSelector:
          matchLabels:
            name: your-app-namespace  # Customize this
```

## Performance Tuning

### Memory Settings

Adjust in `cluster.yaml`:

```yaml
rabbitmq:
  additionalConfig: |
    vm_memory_high_watermark.relative = 0.6  # Adjust based on workload
```

### Queue Policies

Customize HA policies in `definitions.yaml`:

```yaml
"policies": [
  {
    "name": "ha-two",
    "definition": {
      "ha-mode": "exactly",
      "ha-params": 2  # Number of replicas
    }
  }
]
```

### Resource Limits

Adjust based on workload in `cluster.yaml`:

```yaml
resources:
  requests:
    cpu: 1000m      # Increase for high throughput
    memory: 2Gi
  limits:
    cpu: 4000m
    memory: 4Gi
```

## References

- [RabbitMQ Cluster Operator Documentation](https://www.rabbitmq.com/kubernetes/operator/operator-overview.html)
- [RabbitMQ Configuration](https://www.rabbitmq.com/configure.html)
- [RabbitMQ Clustering Guide](https://www.rabbitmq.com/clustering.html)
- [RabbitMQ Production Checklist](https://www.rabbitmq.com/production-checklist.html)
- [RabbitMQ Monitoring](https://www.rabbitmq.com/monitoring.html)
