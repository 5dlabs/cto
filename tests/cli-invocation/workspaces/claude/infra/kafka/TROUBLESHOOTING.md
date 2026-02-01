# Kafka Cluster Troubleshooting Guide

This guide provides solutions to common issues you may encounter with your Strimzi Kafka cluster.

## Quick Diagnostics

Run these commands first to get an overview of the cluster state:

```bash
# Check overall cluster status
kubectl get kafka kafka-cluster -n messaging

# Check all pods
kubectl get pods -n messaging

# Check recent events
kubectl get events -n messaging --sort-by='.lastTimestamp' | tail -20

# Run the validation script
./validate.sh
```

## Common Issues and Solutions

### 1. Pods Not Starting

**Symptoms:**
- Pods stuck in `Pending` or `ContainerCreating` state
- PVCs not binding

**Diagnosis:**
```bash
kubectl describe pod <pod-name> -n messaging
kubectl get pvc -n messaging
kubectl describe pvc <pvc-name> -n messaging
```

**Solutions:**

**Storage class not available:**
```bash
# List available storage classes
kubectl get storageclass

# Update cluster.yaml if needed to use a different storage class
# Change 'class: standard' to your available storage class
```

**Insufficient resources:**
```bash
# Check node resources
kubectl describe nodes | grep -A 5 "Allocated resources"

# Reduce resource requests in cluster.yaml if needed
```

**PVC size too large:**
```bash
# Check storage capacity
kubectl get pv

# Reduce storage size in cluster.yaml if needed
```

### 2. Kafka Cluster Not Ready

**Symptoms:**
- Kafka resource shows `Ready: False`
- Cluster takes very long to start

**Diagnosis:**
```bash
kubectl describe kafka kafka-cluster -n messaging
kubectl logs kafka-cluster-kafka-0 -n messaging -c kafka --tail=100
```

**Solutions:**

**ZooKeeper connection issues:**
```bash
# Check ZooKeeper logs
kubectl logs kafka-cluster-zookeeper-0 -n messaging --tail=100

# Test ZooKeeper connectivity
kubectl exec kafka-cluster-kafka-0 -n messaging -c kafka -- \
  bin/zookeeper-shell.sh kafka-cluster-zookeeper-client:2181 ls /
```

**Broker startup timeout:**
```bash
# Increase startup timeouts in cluster.yaml
# Under kafka.readinessProbe and kafka.livenessProbe:
#   initialDelaySeconds: 30  # Increase from 15
#   timeoutSeconds: 10       # Increase from 5
```

**Configuration errors:**
```bash
# Check for configuration issues in logs
kubectl logs kafka-cluster-kafka-0 -n messaging -c kafka | grep -i error

# Validate Kafka resource
kubectl get kafka kafka-cluster -n messaging -o yaml | kubectl apply --dry-run=client -f -
```

### 3. Topic Creation Failures

**Symptoms:**
- KafkaTopic resources not creating topics
- Topics show as "NotReady"

**Diagnosis:**
```bash
kubectl get kafkatopics -n messaging
kubectl describe kafkatopic <topic-name> -n messaging
kubectl logs deployment/kafka-cluster-entity-operator -n messaging -c topic-operator
```

**Solutions:**

**Topic Operator not running:**
```bash
# Check Entity Operator deployment
kubectl get deployment kafka-cluster-entity-operator -n messaging

# Restart if needed
kubectl rollout restart deployment/kafka-cluster-entity-operator -n messaging
```

**Invalid topic configuration:**
```bash
# Check topic spec for errors
kubectl get kafkatopic <topic-name> -n messaging -o yaml

# Common issues:
# - replicas > number of brokers
# - partitions set to 0
# - invalid retention settings
```

**Manual topic creation:**
```bash
# Create topic manually for testing
kubectl exec kafka-cluster-kafka-0 -n messaging -c kafka -- \
  bin/kafka-topics.sh --create \
  --bootstrap-server localhost:9092 \
  --topic test-topic \
  --partitions 3 \
  --replication-factor 3
```

### 4. Connection Issues from Clients

**Symptoms:**
- Cannot connect to Kafka from applications
- Connection timeouts

**Diagnosis:**
```bash
# Test connectivity from within the cluster
kubectl run kafka-test -n messaging --rm -it --restart=Never \
  --image=quay.io/strimzi/kafka:0.38.0-kafka-3.6.0 -- \
  bin/kafka-broker-api-versions.sh \
  --bootstrap-server kafka-cluster-kafka-bootstrap:9092

# Check services
kubectl get svc -n messaging
kubectl describe svc kafka-cluster-kafka-bootstrap -n messaging
```

**Solutions:**

**Wrong bootstrap server address:**
```bash
# Correct addresses:
# Internal Plain: kafka-cluster-kafka-bootstrap.messaging.svc.cluster.local:9092
# Internal TLS: kafka-cluster-kafka-bootstrap.messaging.svc.cluster.local:9093
# External: Use NodePort service address
```

**Network policy blocking traffic:**
```bash
# Check network policies
kubectl get networkpolicy -n messaging

# Temporarily disable to test
kubectl delete networkpolicy kafka-cluster-network-policy -n messaging

# Re-apply after testing
kubectl apply -f network-policy.yaml
```

**TLS certificate issues:**
```bash
# Extract and verify CA certificate
kubectl get secret kafka-cluster-cluster-ca-cert -n messaging \
  -o jsonpath='{.data.ca\.crt}' | base64 -d > ca.crt

# Verify certificate
openssl x509 -in ca.crt -text -noout
```

### 5. Performance Issues

**Symptoms:**
- Slow message throughput
- High latency
- Consumer lag

**Diagnosis:**
```bash
# Check broker metrics
kubectl exec kafka-cluster-kafka-0 -n messaging -c kafka -- \
  bin/kafka-run-class.sh kafka.tools.JmxTool \
  --object-name kafka.server:type=BrokerTopicMetrics,name=MessagesInPerSec \
  --attributes Count

# Check consumer lag
kubectl exec kafka-cluster-kafka-0 -n messaging -c kafka -- \
  bin/kafka-consumer-groups.sh \
  --bootstrap-server localhost:9092 \
  --describe \
  --all-groups
```

**Solutions:**

**Increase broker resources:**
```yaml
# In cluster.yaml, under kafka.resources:
resources:
  requests:
    memory: 4Gi   # Increase from 2Gi
    cpu: 2000m    # Increase from 1000m
  limits:
    memory: 8Gi   # Increase from 4Gi
    cpu: 4000m    # Increase from 2000m
```

**Tune JVM heap:**
```yaml
# In cluster.yaml, under kafka.jvmOptions:
jvmOptions:
  -Xms: 4096m   # Increase from 2048m
  -Xmx: 4096m   # Increase from 2048m
```

**Optimize topic configuration:**
```bash
# Increase partitions for parallelism
kubectl patch kafkatopic events -n messaging --type merge \
  -p '{"spec":{"partitions":12}}'  # Increase from 6

# Adjust compression
kubectl patch kafkatopic events -n messaging --type merge \
  -p '{"spec":{"config":{"compression.type":"lz4"}}}'
```

**Use faster storage:**
```bash
# Change storage class to SSD-backed storage
# Update cluster.yaml:
#   storage:
#     class: fast-ssd  # Change from 'standard'
```

### 6. Storage Issues

**Symptoms:**
- Pods evicted due to disk pressure
- "No space left on device" errors

**Diagnosis:**
```bash
# Check disk usage
kubectl exec kafka-cluster-kafka-0 -n messaging -c kafka -- df -h

# Check PVC size
kubectl get pvc -n messaging
```

**Solutions:**

**Expand PVC (if storage class supports it):**
```bash
# Edit PVC to increase size
kubectl edit pvc data-kafka-cluster-kafka-0 -n messaging
# Change spec.resources.requests.storage to larger value
```

**Reduce retention:**
```bash
# Decrease log retention
kubectl patch kafkatopic events -n messaging --type merge \
  -p '{"spec":{"config":{"retention.ms":"86400000"}}}'  # 1 day
```

**Clean old segments:**
```bash
# Connect to broker and check log size
kubectl exec -it kafka-cluster-kafka-0 -n messaging -c kafka -- \
  du -sh /var/lib/kafka/data/kafka-log*
```

### 7. Authentication/Authorization Issues

**Symptoms:**
- "Not authorized" errors
- Certificate errors

**Diagnosis:**
```bash
# Check user status
kubectl get kafkauser -n messaging
kubectl describe kafkauser <user-name> -n messaging

# Check user operator logs
kubectl logs deployment/kafka-cluster-entity-operator -n messaging -c user-operator
```

**Solutions:**

**Extract user certificates:**
```bash
# Extract user certificate and key
kubectl get secret kafka-producer -n messaging \
  -o jsonpath='{.data.user\.crt}' | base64 -d > user.crt
kubectl get secret kafka-producer -n messaging \
  -o jsonpath='{.data.user\.key}' | base64 -d > user.key
kubectl get secret kafka-producer -n messaging \
  -o jsonpath='{.data.user\.p12}' | base64 -d > user.p12

# Get password for p12
kubectl get secret kafka-producer -n messaging \
  -o jsonpath='{.data.user\.password}' | base64 -d
```

**Update ACLs:**
```bash
# Add additional ACL to user
kubectl patch kafkauser kafka-producer -n messaging --type merge -p '
{
  "spec": {
    "authorization": {
      "acls": [
        {
          "resource": {
            "type": "topic",
            "name": "new-topic",
            "patternType": "literal"
          },
          "operations": ["Write", "Describe"]
        }
      ]
    }
  }
}'
```

### 8. Upgrade Issues

**Symptoms:**
- Cluster stuck during upgrade
- Version mismatch errors

**Diagnosis:**
```bash
# Check Kafka version
kubectl get kafka kafka-cluster -n messaging -o jsonpath='{.spec.kafka.version}'

# Check actual running version
kubectl exec kafka-cluster-kafka-0 -n messaging -c kafka -- \
  cat /opt/kafka/libs/kafka_*
```

**Solutions:**

**Perform rolling update:**
```bash
# Update version in cluster.yaml
# Then apply changes
kubectl apply -f cluster.yaml

# Monitor rolling update
kubectl rollout status sts kafka-cluster-kafka -n messaging
```

**Rollback if needed:**
```bash
# Revert to previous version in cluster.yaml
kubectl apply -f cluster.yaml
```

## Debug Commands Reference

### Viewing Logs

```bash
# Kafka broker logs
kubectl logs kafka-cluster-kafka-0 -n messaging -c kafka --tail=100 -f

# ZooKeeper logs
kubectl logs kafka-cluster-zookeeper-0 -n messaging --tail=100 -f

# Entity Operator logs
kubectl logs deployment/kafka-cluster-entity-operator -n messaging -c topic-operator -f
kubectl logs deployment/kafka-cluster-entity-operator -n messaging -c user-operator -f

# All logs from a specific pod
kubectl logs kafka-cluster-kafka-0 -n messaging --all-containers=true
```

### Executing Commands in Pods

```bash
# Access broker shell
kubectl exec -it kafka-cluster-kafka-0 -n messaging -c kafka -- bash

# List topics
kubectl exec kafka-cluster-kafka-0 -n messaging -c kafka -- \
  bin/kafka-topics.sh --list --bootstrap-server localhost:9092

# Describe topic
kubectl exec kafka-cluster-kafka-0 -n messaging -c kafka -- \
  bin/kafka-topics.sh --describe --topic events --bootstrap-server localhost:9092

# Check consumer groups
kubectl exec kafka-cluster-kafka-0 -n messaging -c kafka -- \
  bin/kafka-consumer-groups.sh --list --bootstrap-server localhost:9092

# Test producer
kubectl exec -it kafka-cluster-kafka-0 -n messaging -c kafka -- \
  bin/kafka-console-producer.sh --topic events --bootstrap-server localhost:9092

# Test consumer
kubectl exec -it kafka-cluster-kafka-0 -n messaging -c kafka -- \
  bin/kafka-console-consumer.sh --topic events --from-beginning --bootstrap-server localhost:9092
```

### Monitoring Cluster Health

```bash
# Check cluster state
kubectl get kafka,kafkatopic,kafkauser -n messaging

# Check all resources
kubectl get all -n messaging

# Check resource usage
kubectl top pods -n messaging

# Check PVC usage
kubectl exec kafka-cluster-kafka-0 -n messaging -c kafka -- df -h | grep kafka
```

## Getting Help

If you're still experiencing issues:

1. **Collect diagnostic information:**
   ```bash
   kubectl get all -n messaging > diagnostics.txt
   kubectl describe kafka kafka-cluster -n messaging >> diagnostics.txt
   kubectl logs kafka-cluster-kafka-0 -n messaging -c kafka --tail=200 >> diagnostics.txt
   ```

2. **Check Strimzi documentation:**
   - [Strimzi Documentation](https://strimzi.io/docs/operators/latest/overview.html)
   - [Strimzi GitHub Issues](https://github.com/strimzi/strimzi-kafka-operator/issues)

3. **Check Kafka documentation:**
   - [Apache Kafka Documentation](https://kafka.apache.org/documentation/)

4. **Community support:**
   - Strimzi Slack: [Join here](https://slack.cncf.io/)
   - CNCF Slack #strimzi channel
