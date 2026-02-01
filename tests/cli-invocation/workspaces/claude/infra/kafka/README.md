# Kafka Cluster with Strimzi Operator

This directory contains Kubernetes manifests for deploying a production-ready Apache Kafka cluster using the Strimzi operator.

## Architecture Overview

- **Kafka Brokers**: 3 replicas for high availability
- **ZooKeeper**: 3 nodes for cluster coordination
- **Kafka Version**: 3.6.0
- **Namespace**: messaging
- **Storage**: Persistent volumes with 50Gi per Kafka broker, 5Gi per ZooKeeper node

## Prerequisites

1. Kubernetes cluster (1.19+)
2. Strimzi Operator installed in the cluster
3. kubectl configured to access your cluster
4. Storage class available (default: `standard`)

## Installing Strimzi Operator

If Strimzi is not already installed, deploy it first:

```bash
# Install Strimzi operator
kubectl create namespace kafka
kubectl create -f 'https://strimzi.io/install/latest?namespace=kafka' -n kafka

# Wait for the operator to be ready
kubectl wait --for=condition=ready pod -l name=strimzi-cluster-operator -n kafka --timeout=300s
```

## Deployment

### Deploy using kubectl

```bash
# Apply all manifests
kubectl apply -f namespace.yaml
kubectl apply -f metrics-config.yaml
kubectl apply -f cluster.yaml
kubectl apply -f topics.yaml
kubectl apply -f users.yaml
kubectl apply -f pod-disruption-budget.yaml
kubectl apply -f network-policy.yaml
```

### Deploy using Kustomize

```bash
# Deploy all resources
kubectl apply -k .

# Or specify the directory
kubectl apply -k /workspace/infra/kafka/
```

## Verification

Check the status of your Kafka cluster:

```bash
# Check Kafka cluster status
kubectl get kafka kafka-cluster -n messaging

# Check Kafka pods
kubectl get pods -n messaging -l strimzi.io/cluster=kafka-cluster

# Check topics
kubectl get kafkatopics -n messaging

# Check users
kubectl get kafkausers -n messaging

# View cluster details
kubectl describe kafka kafka-cluster -n messaging
```

## Accessing Kafka

### Internal Access (within Kubernetes)

**Bootstrap servers:**
- Plain: `kafka-cluster-kafka-bootstrap.messaging.svc.cluster.local:9092`
- TLS: `kafka-cluster-kafka-bootstrap.messaging.svc.cluster.local:9093`

### External Access

External access is configured via NodePort on port 9094 with TLS enabled.

```bash
# Get the external address
kubectl get service kafka-cluster-kafka-external-bootstrap -n messaging
```

## Topics

Three sample topics are created:

1. **events** - 6 partitions, 7-day retention
2. **commands** - 3 partitions, 3-day retention
3. **logs** - 12 partitions, 1-day retention

### Creating Additional Topics

```bash
# Create a new topic using kubectl
kubectl apply -f - <<EOF
apiVersion: kafka.strimzi.io/v1beta2
kind: KafkaTopic
metadata:
  name: my-topic
  namespace: messaging
  labels:
    strimzi.io/cluster: kafka-cluster
spec:
  partitions: 3
  replicas: 3
  config:
    retention.ms: 604800000
    min.insync.replicas: 2
EOF
```

## Users and Authentication

Three sample users are configured with TLS authentication:

1. **kafka-producer** - Write access to events and commands topics
2. **kafka-consumer** - Read access to all topics
3. **kafka-admin** - Full administrative access

### Accessing User Credentials

User credentials are stored as Kubernetes secrets:

```bash
# Extract producer certificate
kubectl get secret kafka-producer -n messaging -o jsonpath='{.data.user\.crt}' | base64 -d > producer.crt
kubectl get secret kafka-producer -n messaging -o jsonpath='{.data.user\.key}' | base64 -d > producer.key

# Extract CA certificate
kubectl get secret kafka-cluster-cluster-ca-cert -n messaging -o jsonpath='{.data.ca\.crt}' | base64 -d > ca.crt
```

## Monitoring

The cluster is configured with JMX Prometheus exporters for both Kafka and ZooKeeper.

**Metrics endpoints:**
- Kafka: `http://<pod-ip>:9404/metrics`
- ZooKeeper: `http://<pod-ip>:9404/metrics`

### Prometheus ServiceMonitor

If using Prometheus Operator, create a ServiceMonitor:

```bash
kubectl apply -f - <<EOF
apiVersion: monitoring.coreos.com/v1
kind: ServiceMonitor
metadata:
  name: kafka-cluster-metrics
  namespace: messaging
spec:
  selector:
    matchLabels:
      strimzi.io/cluster: kafka-cluster
  endpoints:
  - port: tcp-prometheus
    interval: 30s
EOF
```

## Resource Limits

### Kafka Brokers
- **Requests**: 1 CPU, 2Gi memory
- **Limits**: 2 CPU, 4Gi memory
- **JVM Heap**: 2Gi

### ZooKeeper
- **Requests**: 250m CPU, 512Mi memory
- **Limits**: 500m CPU, 1Gi memory
- **JVM Heap**: 512Mi

### Entity Operator
- **Requests**: 100m CPU, 256Mi memory per operator
- **Limits**: 500m CPU, 512Mi memory per operator

## High Availability Features

- **Replication Factor**: 3 for all topics
- **Min In-Sync Replicas**: 2
- **Pod Disruption Budgets**: Max 1 unavailable pod during disruptions
- **Persistent Storage**: Data survives pod restarts
- **Multiple Availability Zones**: Configure pod anti-affinity for distribution

## Configuration Details

### Kafka Broker Settings

- **Auto-create topics**: Enabled with 3 default partitions
- **Compression**: Producer-controlled
- **Log retention**: 7 days (168 hours)
- **Replication factor**: 3 (for HA)
- **Min ISR**: 2 (for data durability)

### Listeners

1. **Plain** (9092): Internal, no TLS - for development
2. **TLS** (9093): Internal, TLS enabled - for production
3. **External** (9094): NodePort, TLS enabled - for external clients

## Troubleshooting

### Check pod logs

```bash
# Kafka broker logs
kubectl logs kafka-cluster-kafka-0 -n messaging -c kafka

# ZooKeeper logs
kubectl logs kafka-cluster-zookeeper-0 -n messaging

# Entity operator logs
kubectl logs deployment/kafka-cluster-entity-operator -n messaging -c topic-operator
kubectl logs deployment/kafka-cluster-entity-operator -n messaging -c user-operator
```

### Check cluster status

```bash
# Describe the Kafka resource
kubectl describe kafka kafka-cluster -n messaging

# Check for events
kubectl get events -n messaging --sort-by='.lastTimestamp'
```

### Common Issues

1. **Pods not starting**: Check storage class availability and PVC binding
2. **Connection timeouts**: Verify network policies and service endpoints
3. **Topic creation failing**: Check entity operator logs and RBAC permissions

## Scaling

### Scale Kafka brokers

```bash
kubectl patch kafka kafka-cluster -n messaging --type merge -p '{"spec":{"kafka":{"replicas":5}}}'
```

### Scale ZooKeeper ensemble

```bash
kubectl patch kafka kafka-cluster -n messaging --type merge -p '{"spec":{"zookeeper":{"replicas":5}}}'
```

**Note**: Only scale to odd numbers (3, 5, 7) for ZooKeeper quorum.

## Backup and Recovery

### Backup topics configuration

```bash
kubectl get kafkatopics -n messaging -o yaml > topics-backup.yaml
```

### Backup user configuration

```bash
kubectl get kafkausers -n messaging -o yaml > users-backup.yaml
```

## Security Considerations

1. **TLS Encryption**: Enabled for external and internal TLS listeners
2. **Authentication**: User authentication via TLS certificates
3. **Authorization**: ACL-based authorization for topics and groups
4. **Network Policies**: Restrict traffic to/from Kafka pods
5. **Pod Security**: Consider using Pod Security Standards

## Performance Tuning

For production workloads, consider adjusting:

- `num.network.threads` and `num.io.threads` for broker performance
- `replica.fetch.max.bytes` for replication throughput
- `log.segment.bytes` and `log.index.size.max.bytes` for disk I/O
- Storage class to use high-performance SSDs

## Cleanup

To remove the Kafka cluster:

```bash
# Delete using kustomize
kubectl delete -k .

# Or delete individual resources
kubectl delete kafka kafka-cluster -n messaging
kubectl delete kafkatopics --all -n messaging
kubectl delete kafkausers --all -n messaging
kubectl delete namespace messaging
```

**Warning**: This will delete all data. Ensure you have backups if needed.

## References

- [Strimzi Documentation](https://strimzi.io/docs/operators/latest/overview.html)
- [Apache Kafka Documentation](https://kafka.apache.org/documentation/)
- [Kafka Configuration Reference](https://kafka.apache.org/documentation/#configuration)
