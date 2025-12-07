# Database Operators for CTO Platform

This directory contains Kubernetes operators for databases that can be distributed as part of the CTO Platform-in-a-Box product.

## License Compliance

All operators in this directory are **Apache 2.0 licensed** and safe to distribute commercially.

| Operator | License | GitHub Stars | Maintained By |
|----------|---------|--------------|---------------|
| Strimzi (Kafka) | Apache 2.0 | 5.6k | CNCF Incubating |
| Altinity ClickHouse | Apache 2.0 | 2.3k | Altinity |
| OpenSearch K8s | Apache 2.0 | 502 | OpenSearch Project |

## Quick Start

### Install All Operators

```bash
./install-operators.sh all
```

### Install Individual Operators

```bash
./install-operators.sh strimzi     # Apache Kafka
./install-operators.sh clickhouse  # ClickHouse OLAP
./install-operators.sh opensearch  # OpenSearch
```

### Deploy Test Instances

```bash
./install-operators.sh instances
```

### Check Status

```bash
./install-operators.sh status
```

## Operators Overview

### Strimzi - Apache Kafka Operator

**Use Case**: Event streaming, async communication, CDC, log aggregation

**AWS Replacement**: Kinesis, MSK

**Features**:
- Kafka cluster management
- Topic and user management
- Kafka Connect support
- Mirror Maker for replication
- Schema Registry integration

```yaml
apiVersion: kafka.strimzi.io/v1beta2
kind: Kafka
metadata:
  name: my-kafka
spec:
  kafka:
    version: 3.9.0
    replicas: 3
  zookeeper:
    replicas: 3
```

### Altinity ClickHouse Operator

**Use Case**: OLAP analytics, real-time dashboards, log analytics

**AWS Replacement**: Redshift, Athena

**Features**:
- ClickHouse cluster management
- Automatic schema propagation
- Replication and sharding
- Integration with Grafana
- ZooKeeper/Keeper management

```yaml
apiVersion: clickhouse.altinity.com/v1
kind: ClickHouseInstallation
metadata:
  name: my-clickhouse
spec:
  configuration:
    clusters:
      - name: default
        layout:
          shardsCount: 2
          replicasCount: 2
```

### OpenSearch Kubernetes Operator

**Use Case**: Full-text search, log analytics, observability

**AWS Replacement**: OpenSearch Service, Elasticsearch Service

**Features**:
- OpenSearch cluster management
- OpenSearch Dashboards deployment
- Rolling upgrades
- Security configuration
- Index lifecycle management

```yaml
apiVersion: opensearch.opster.io/v1
kind: OpenSearchCluster
metadata:
  name: my-opensearch
spec:
  general:
    version: 2.19.0
  nodePools:
    - component: masters
      replicas: 3
      roles: [master, data, ingest]
```

## Test Instances

The `test-instances/` directory contains minimal configurations for testing on Kind:

- `kafka-cluster.yaml` - Single-node Kafka cluster
- `clickhouse-cluster.yaml` - Single-node ClickHouse
- `opensearch-cluster.yaml` - Single-node OpenSearch with Dashboards

## Resource Requirements

### Minimum (Kind/Development)

| Component | CPU Request | Memory Request | Storage |
|-----------|-------------|----------------|---------|
| Kafka (1 node) | 250m | 512Mi | 5Gi |
| ZooKeeper (1 node) | 100m | 256Mi | 2Gi |
| ClickHouse (1 node) | 250m | 512Mi | 5Gi |
| OpenSearch (1 node) | 250m | 512Mi | 5Gi |
| **Total** | **850m** | **1.75Gi** | **17Gi** |

### Production (Recommended)

| Component | CPU Request | Memory Request | Storage |
|-----------|-------------|----------------|---------|
| Kafka (3 nodes) | 2 cores | 4Gi | 100Gi |
| ZooKeeper (3 nodes) | 500m | 1Gi | 10Gi |
| ClickHouse (2 shards x 2 replicas) | 4 cores | 8Gi | 500Gi |
| OpenSearch (3 nodes) | 2 cores | 4Gi | 200Gi |

## Integration with CTO Platform

These databases integrate with the CTO Platform for:

1. **Agent Telemetry** - ClickHouse stores agent execution metrics
2. **Event Streaming** - Kafka handles async agent communication
3. **Log Search** - OpenSearch provides log aggregation and search
4. **CDC Pipelines** - Kafka Connect for database change capture

## ArgoCD Deployment

ArgoCD Application manifests are available at `/infra/gitops/applications/`:

- `strimzi-kafka-operator.yaml` - Apache Kafka (Strimzi 0.49.0)
- `clickhouse-operator.yaml` - ClickHouse OLAP (Altinity 0.25.5)
- `opensearch-operator.yaml` - OpenSearch search/analytics (2.8.0)

To deploy via ArgoCD:
```bash
kubectl apply -f /infra/gitops/applications/strimzi-kafka-operator.yaml
kubectl apply -f /infra/gitops/applications/clickhouse-operator.yaml
kubectl apply -f /infra/gitops/applications/opensearch-operator.yaml
```

## Test Results

| Operator | Status | Notes |
|----------|--------|-------|
| Strimzi Kafka | ✅ Working | KRaft mode (ZK-less), Kafka 4.0.0 |
| Altinity ClickHouse | ✅ Working | v24.8, requires password auth |
| OpenSearch | ✅ Running | Security config needs production tuning |

## Troubleshooting

### Check Operator Logs

```bash
# Strimzi
kubectl logs -n strimzi -l name=strimzi-cluster-operator

# ClickHouse
kubectl logs -n clickhouse-operator -l app=clickhouse-operator

# OpenSearch
kubectl logs -n opensearch-operator -l app.kubernetes.io/name=opensearch-operator
```

### Common Issues

1. **CRDs not found**: Ensure operator is installed before creating instances
2. **Storage issues**: Check StorageClass exists (`local-path` for Kind)
3. **Resource limits**: Increase if pods are OOMKilled

