# RabbitMQ Cluster Deployment Guide

## Quick Start

### Prerequisites Check

Before deploying, ensure:

1. **RabbitMQ Cluster Operator is installed**:
   ```bash
   kubectl get crd rabbitmqclusters.rabbitmq.com
   ```

   If not installed:
   ```bash
   kubectl apply -f "https://github.com/rabbitmq/cluster-operator/releases/latest/download/cluster-operator.yml"
   ```

2. **Default StorageClass exists**:
   ```bash
   kubectl get storageclass
   ```

3. **Sufficient cluster resources**:
   - At least 3 worker nodes (for anti-affinity)
   - 1.5Gi+ memory per node
   - 10Gi+ storage per node

### Installation Steps

#### Option 1: Deploy with Kustomize (Recommended)

```bash
# Validate configuration
kubectl kustomize /workspace/infra/rabbitmq/

# Deploy all resources
kubectl apply -k /workspace/infra/rabbitmq/

# Watch deployment progress
kubectl get pods -n messaging -w
```

#### Option 2: Deploy Individually

```bash
# Step 1: Create namespace and RBAC
kubectl apply -f /workspace/infra/rabbitmq/namespace.yaml
kubectl apply -f /workspace/infra/rabbitmq/rbac.yaml

# Step 2: Create configuration
kubectl apply -f /workspace/infra/rabbitmq/configmap.yaml
kubectl apply -f /workspace/infra/rabbitmq/definitions.yaml

# Step 3: Deploy cluster
kubectl apply -f /workspace/infra/rabbitmq/cluster.yaml

# Step 4: Create services
kubectl apply -f /workspace/infra/rabbitmq/service.yaml

# Step 5: Apply security and reliability
kubectl apply -f /workspace/infra/rabbitmq/networkpolicy.yaml
kubectl apply -f /workspace/infra/rabbitmq/poddisruptionbudget.yaml

# Step 6: Setup monitoring (optional)
kubectl apply -f /workspace/infra/rabbitmq/monitoring.yaml
```

### Verify Deployment

```bash
# Check all resources
kubectl get all -n messaging

# Expected output should show:
# - 3 pods: rabbitmq-cluster-server-0, rabbitmq-cluster-server-1, rabbitmq-cluster-server-2
# - 4 services: amqp, management, metrics, nodes
# - 1 statefulset: rabbitmq-cluster-server
# - 3 PVCs: persistence-rabbitmq-cluster-server-{0,1,2}

# Check cluster health
kubectl exec -n messaging rabbitmq-cluster-server-0 -- rabbitmq-diagnostics check_running
kubectl exec -n messaging rabbitmq-cluster-server-0 -- rabbitmq-diagnostics cluster_status

# Check all nodes are clustered
kubectl exec -n messaging rabbitmq-cluster-server-0 -- rabbitmqctl cluster_status
```

### Access RabbitMQ

#### Management UI

```bash
# Port forward
kubectl port-forward -n messaging svc/rabbitmq-cluster-management 15672:15672

# Access at: http://localhost:15672
# Default credentials: admin / changeme123
```

#### AMQP Connection

```bash
# Port forward
kubectl port-forward -n messaging svc/rabbitmq-cluster-amqp 5672:5672

# Connection string: amqp://admin:changeme123@localhost:5672/
# Or for app-user: amqp://app-user:changeme456@localhost:5672/production
```

#### Prometheus Metrics

```bash
# Port forward
kubectl port-forward -n messaging svc/rabbitmq-cluster-metrics 15692:15692

# Metrics at: http://localhost:15692/metrics
```

## Post-Deployment Configuration

### 1. Change Default Passwords (CRITICAL)

```bash
# Generate strong passwords
ADMIN_PASS=$(openssl rand -base64 32)
APP_PASS=$(openssl rand -base64 32)
MONITORING_PASS=$(openssl rand -base64 32)

# Update definitions secret
kubectl create secret generic rabbitmq-cluster-definitions \
  --from-literal=definitions.json="$(cat <<EOF
{
  "users": [
    {"name": "admin", "password": "${ADMIN_PASS}", "tags": "administrator"},
    {"name": "app-user", "password": "${APP_PASS}", "tags": "management"},
    {"name": "monitoring", "password": "${MONITORING_PASS}", "tags": "monitoring"}
  ],
  ...
}
EOF
)" \
  --dry-run=client -o yaml | kubectl apply -n messaging -f -

# Restart pods to pick up new credentials
kubectl delete pod -n messaging -l app.kubernetes.io/name=rabbitmq-cluster
```

### 2. Enable TLS (Production)

```bash
# Create TLS certificate (example with cert-manager)
cat <<EOF | kubectl apply -f -
apiVersion: cert-manager.io/v1
kind: Certificate
metadata:
  name: rabbitmq-cluster-tls
  namespace: messaging
spec:
  secretName: rabbitmq-tls-secret
  issuerRef:
    name: letsencrypt-prod
    kind: ClusterIssuer
  dnsNames:
    - rabbitmq-cluster.messaging.svc.cluster.local
    - "*.rabbitmq-cluster-nodes.messaging.svc.cluster.local"
EOF

# Update cluster.yaml to enable TLS (uncomment tls section)
# Then reapply:
kubectl apply -f /workspace/infra/rabbitmq/cluster.yaml
```

### 3. Configure Ingress for Management UI

```bash
cat <<EOF | kubectl apply -f -
apiVersion: networking.k8s.io/v1
kind: Ingress
metadata:
  name: rabbitmq-management
  namespace: messaging
  annotations:
    cert-manager.io/cluster-issuer: letsencrypt-prod
    nginx.ingress.kubernetes.io/backend-protocol: "HTTP"
    nginx.ingress.kubernetes.io/auth-type: basic
    nginx.ingress.kubernetes.io/auth-secret: rabbitmq-basic-auth
spec:
  ingressClassName: nginx
  tls:
    - hosts:
        - rabbitmq.yourdomain.com
      secretName: rabbitmq-management-tls
  rules:
    - host: rabbitmq.yourdomain.com
      http:
        paths:
          - path: /
            pathType: Prefix
            backend:
              service:
                name: rabbitmq-cluster-management
                port:
                  number: 15672
EOF
```

## Testing the Cluster

### Basic Connectivity Test

```bash
# Install Python client (if not available)
pip install pika

# Test script
cat > test_rabbitmq.py <<'EOF'
import pika
import sys

# Connection parameters
credentials = pika.PlainCredentials('admin', 'changeme123')
parameters = pika.ConnectionParameters(
    host='localhost',
    port=5672,
    virtual_host='/production',
    credentials=credentials
)

try:
    # Connect
    connection = pika.BlockingConnection(parameters)
    channel = connection.channel()

    # Declare queue
    channel.queue_declare(queue='test_queue', durable=True)

    # Publish message
    channel.basic_publish(
        exchange='',
        routing_key='test_queue',
        body='Hello RabbitMQ Cluster!'
    )
    print("✓ Message published successfully")

    # Consume message
    method, properties, body = channel.basic_get(queue='test_queue', auto_ack=True)
    if body:
        print(f"✓ Message received: {body.decode()}")

    # Cleanup
    channel.queue_delete(queue='test_queue')
    connection.close()
    print("✓ Test completed successfully")

except Exception as e:
    print(f"✗ Test failed: {e}")
    sys.exit(1)
EOF

# Run test (with port forward active)
python test_rabbitmq.py
```

### High Availability Test

```bash
# 1. Create test queue with HA policy
kubectl exec -n messaging rabbitmq-cluster-server-0 -- rabbitmqadmin \
  -u admin -p changeme123 -V /production \
  declare queue name=ha_test_queue durable=true

# 2. Publish messages
for i in {1..100}; do
  kubectl exec -n messaging rabbitmq-cluster-server-0 -- rabbitmqadmin \
    -u admin -p changeme123 -V /production \
    publish routing_key=ha_test_queue payload="Message $i"
done

# 3. Verify replication
kubectl exec -n messaging rabbitmq-cluster-server-0 -- rabbitmqctl list_queues -p /production \
  name messages slave_pids synchronised_slave_pids

# 4. Kill a pod and verify messages still available
kubectl delete pod -n messaging rabbitmq-cluster-server-1
sleep 10

# 5. Check messages still exist
kubectl exec -n messaging rabbitmq-cluster-server-0 -- rabbitmqadmin \
  -u admin -p changeme123 -V /production \
  list queues name messages
```

## Monitoring Setup

### Grafana Dashboard

```bash
# Import RabbitMQ dashboard (ID: 10991)
# Or use this JSON URL:
# https://grafana.com/grafana/dashboards/10991-rabbitmq-cluster/
```

### Check Prometheus Metrics

```bash
# Verify ServiceMonitor is detected
kubectl get servicemonitor -n messaging

# Check Prometheus targets (if using Prometheus Operator)
kubectl port-forward -n monitoring svc/prometheus-k8s 9090:9090
# Visit: http://localhost:9090/targets
# Look for: messaging/rabbitmq-cluster-metrics
```

### Test Alerts

```bash
# Trigger memory alert (for testing)
kubectl exec -n messaging rabbitmq-cluster-server-0 -- \
  bash -c 'head -c 1G </dev/zero | rabbitmqadmin -u admin -p changeme123 publish routing_key=test payload=-'

# Check AlertManager
kubectl port-forward -n monitoring svc/alertmanager-main 9093:9093
# Visit: http://localhost:9093
```

## Scaling Operations

### Scale Up

```bash
# Scale to 5 nodes
kubectl patch rabbitmqcluster rabbitmq-cluster -n messaging \
  --type='merge' -p '{"spec":{"replicas":5}}'

# Wait for new pods
kubectl rollout status statefulset/rabbitmq-cluster-server -n messaging

# Verify cluster
kubectl exec -n messaging rabbitmq-cluster-server-0 -- rabbitmqctl cluster_status
```

### Scale Down

```bash
# Scale to 3 nodes (safely)
# First, ensure no critical queues on nodes to be removed
kubectl exec -n messaging rabbitmq-cluster-server-4 -- rabbitmqctl list_queues

# Then scale
kubectl patch rabbitmqcluster rabbitmq-cluster -n messaging \
  --type='merge' -p '{"spec":{"replicas":3}}'
```

## Backup and Restore

### Backup Definitions

```bash
# Export definitions
kubectl exec -n messaging rabbitmq-cluster-server-0 -- \
  rabbitmqctl export_definitions /tmp/definitions.json

# Copy to local
kubectl cp messaging/rabbitmq-cluster-server-0:/tmp/definitions.json \
  ./rabbitmq-backup-$(date +%Y%m%d).json

# Backup to S3 (example)
aws s3 cp ./rabbitmq-backup-$(date +%Y%m%d).json \
  s3://my-backup-bucket/rabbitmq/
```

### Backup Messages (using shovel)

```bash
# Create shovel to backup queue
kubectl exec -n messaging rabbitmq-cluster-server-0 -- rabbitmqctl \
  set_parameter shovel backup-shovel \
  '{"src-uri":"amqp://","src-queue":"source_queue",
    "dest-uri":"amqp://backup-server","dest-queue":"backup_queue"}'
```

### Restore Definitions

```bash
# Copy backup to pod
kubectl cp ./rabbitmq-backup-20260131.json \
  messaging/rabbitmq-cluster-server-0:/tmp/definitions.json

# Import definitions
kubectl exec -n messaging rabbitmq-cluster-server-0 -- \
  rabbitmqctl import_definitions /tmp/definitions.json
```

### Disaster Recovery

```bash
# Full cluster restore from backup
# 1. Delete existing cluster
kubectl delete rabbitmqcluster rabbitmq-cluster -n messaging

# 2. Delete PVCs (WARNING: data loss)
kubectl delete pvc -n messaging -l app.kubernetes.io/name=rabbitmq-cluster

# 3. Recreate cluster
kubectl apply -f /workspace/infra/rabbitmq/cluster.yaml

# 4. Wait for cluster ready
kubectl wait --for=condition=Ready pod/rabbitmq-cluster-server-0 -n messaging --timeout=300s

# 5. Restore definitions
kubectl cp ./rabbitmq-backup-20260131.json \
  messaging/rabbitmq-cluster-server-0:/tmp/definitions.json
kubectl exec -n messaging rabbitmq-cluster-server-0 -- \
  rabbitmqctl import_definitions /tmp/definitions.json
```

## Troubleshooting

### Pod Startup Issues

```bash
# Check pod events
kubectl describe pod -n messaging rabbitmq-cluster-server-0

# Check logs
kubectl logs -n messaging rabbitmq-cluster-server-0 --tail=100

# Check PVC status
kubectl get pvc -n messaging
kubectl describe pvc -n messaging persistence-rabbitmq-cluster-server-0
```

### Cluster Formation Problems

```bash
# Check cluster status
kubectl exec -n messaging rabbitmq-cluster-server-0 -- rabbitmqctl cluster_status

# Check peer discovery
kubectl logs -n messaging rabbitmq-cluster-server-0 | grep -i "peer discovery"

# Check DNS resolution
kubectl exec -n messaging rabbitmq-cluster-server-0 -- \
  nslookup rabbitmq-cluster-nodes.messaging.svc.cluster.local

# Manually reset node (last resort)
kubectl exec -n messaging rabbitmq-cluster-server-1 -- rabbitmqctl stop_app
kubectl exec -n messaging rabbitmq-cluster-server-1 -- rabbitmqctl reset
kubectl exec -n messaging rabbitmq-cluster-server-1 -- rabbitmqctl start_app
```

### Memory Issues

```bash
# Check memory usage
kubectl exec -n messaging rabbitmq-cluster-server-0 -- \
  rabbitmq-diagnostics memory_breakdown

# Check memory alarms
kubectl exec -n messaging rabbitmq-cluster-server-0 -- \
  rabbitmqctl status | grep -A 10 "Memory"

# Clear memory alarm (if false positive)
kubectl exec -n messaging rabbitmq-cluster-server-0 -- \
  rabbitmqctl set_vm_memory_high_watermark 0.7
```

### Network Connectivity

```bash
# Test AMQP connection from another pod
kubectl run -it --rm debug --image=alpine --restart=Never -- sh
apk add --no-cache python3 py3-pip
pip3 install pika
python3 -c "import pika; pika.BlockingConnection(pika.ConnectionParameters('rabbitmq-cluster-amqp.messaging.svc.cluster.local'))"

# Check NetworkPolicy
kubectl describe networkpolicy -n messaging

# Temporarily disable NetworkPolicy for testing
kubectl delete networkpolicy rabbitmq-cluster -n messaging
# Re-enable after testing:
kubectl apply -f /workspace/infra/rabbitmq/networkpolicy.yaml
```

### Performance Issues

```bash
# Check queue stats
kubectl exec -n messaging rabbitmq-cluster-server-0 -- \
  rabbitmqctl list_queues name messages consumers memory

# Check connection stats
kubectl exec -n messaging rabbitmq-cluster-server-0 -- \
  rabbitmqctl list_connections name channels

# Check for blocked connections
kubectl exec -n messaging rabbitmq-cluster-server-0 -- \
  rabbitmqctl list_connections state | grep -i blocked

# Enable detailed logging
kubectl exec -n messaging rabbitmq-cluster-server-0 -- \
  rabbitmqctl set_log_level debug
```

## Maintenance

### Update RabbitMQ Version

```bash
# Edit cluster.yaml and update image tag
# Example: rabbitmq:3.12-management -> rabbitmq:3.13-management

# Apply update
kubectl apply -f /workspace/infra/rabbitmq/cluster.yaml

# Monitor rolling update
kubectl rollout status statefulset/rabbitmq-cluster-server -n messaging

# Verify version
kubectl exec -n messaging rabbitmq-cluster-server-0 -- rabbitmqctl version
```

### Drain Node for Maintenance

```bash
# Move queue masters off node-0
kubectl exec -n messaging rabbitmq-cluster-server-0 -- \
  rabbitmqctl sync_queue <queue_name>

# Delete pod (StatefulSet will recreate)
kubectl delete pod -n messaging rabbitmq-cluster-server-0
```

### Clean Old Logs

```bash
# Rotate logs
kubectl exec -n messaging rabbitmq-cluster-server-0 -- \
  rabbitmqctl rotate_logs
```

## Security Checklist

- [ ] Changed all default passwords
- [ ] Enabled TLS for AMQP connections
- [ ] Configured TLS for Management UI
- [ ] Applied NetworkPolicy restrictions
- [ ] Limited RBAC permissions
- [ ] Configured PodSecurityPolicy/PodSecurityStandards
- [ ] Enabled audit logging
- [ ] Configured firewall rules
- [ ] Set up certificate rotation
- [ ] Implemented secrets rotation

## Production Readiness Checklist

- [ ] RabbitMQ Cluster Operator installed
- [ ] Cluster deployed with 3+ nodes
- [ ] Persistent storage configured
- [ ] Anti-affinity rules applied
- [ ] Resource limits configured
- [ ] PodDisruptionBudget created
- [ ] Monitoring configured (Prometheus)
- [ ] Alerting rules configured
- [ ] Backup strategy implemented
- [ ] TLS enabled
- [ ] NetworkPolicy applied
- [ ] Default credentials changed
- [ ] HA policies configured
- [ ] Load testing completed
- [ ] Disaster recovery tested
- [ ] Documentation updated
- [ ] Runbook created
- [ ] On-call rotation assigned

## Support

For issues and questions:

- RabbitMQ Documentation: https://www.rabbitmq.com/documentation.html
- RabbitMQ Cluster Operator: https://github.com/rabbitmq/cluster-operator
- Community: https://groups.google.com/forum/#!forum/rabbitmq-users
- Commercial Support: https://www.rabbitmq.com/commercial-offerings.html
