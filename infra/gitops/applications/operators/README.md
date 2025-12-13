# Kubernetes Operators

This directory contains ArgoCD Application manifests for Kubernetes operators that provide
declarative management of various infrastructure components.

## License Compliance

All operators are selected for compatibility with proprietary distribution. We prioritize:

1. **Apache 2.0** - Preferred, no restrictions on proprietary use
2. **MIT/BSD** - Permissive, no restrictions
3. **MPL 2.0** - Weak copyleft, file-level disclosure only

All storage operators use permissive licenses that allow proprietary distribution.

## Operator Categories

### AI/ML
- **kubeai.yaml** - AI Inference (LLM, VLM, Embeddings, Speech-to-Text) - Apache 2.0
- **nvidia-gpu-operator.yaml** - GPU provisioning and management - Apache 2.0

### Storage
- **seaweedfs-operator.yaml** - S3-compatible object storage - Apache 2.0

> **Note**: Block storage (Mayastor) is in `applications/storage/` directory.

### Databases
- **cloudnative-pg-operator.yaml** - PostgreSQL - Apache 2.0
- **redis-operator.yaml** - Redis/Valkey - Apache 2.0 (uses BSD-3 Valkey images)
- **percona-mongodb-operator.yaml** - MongoDB - Apache 2.0
- **percona-mysql-operator.yaml** - MySQL - Apache 2.0
- **scylladb-operator.yaml** - ScyllaDB (Cassandra-compatible) - Apache 2.0
- **clickhouse-operator.yaml** - ClickHouse analytics - Apache 2.0
- **questdb-operator.yaml** - Time series database - Apache 2.0
- **opensearch-operator.yaml** - OpenSearch - Apache 2.0

### Messaging
- **strimzi-kafka-operator.yaml** - Apache Kafka - Apache 2.0
- **rabbitmq-operator.yaml** - RabbitMQ - MPL 2.0
- **nats.yaml** - NATS messaging - Apache 2.0

### Observability
- **opentelemetry-operator.yaml** - OpenTelemetry - Apache 2.0
- **jaeger-operator.yaml** - Distributed tracing - Apache 2.0

### Container Registry
- **harbor-operator.yaml** - Container registry with scanning - Apache 2.0

### Identity/Auth
- **keycloak-operator.yaml** - Identity and access management - Apache 2.0

### Workflow
- **temporal-operator.yaml** - Workflow orchestration - Apache 2.0

## Adding New Operators

Before adding a new operator:

1. **Verify License**: Check the operator's license on GitHub/Artifact Hub
2. **Avoid AGPL/GPL**: These require source disclosure
3. **Document License**: Add license info in the YAML header comment
4. **Update This README**: Add to the appropriate category

## Sync Wave Order

Operators use ArgoCD sync waves for dependency management:

- Wave `-10`: Storage (Mayastor - in `applications/storage/`)
- Wave `-3`: GPU Operator
- Wave `-2`: Core operators (OpenTelemetry, cert-manager deps)
- Wave `-1`: Observability (Jaeger)
- Wave `0`: Default (most operators)
- Wave `1`: Application-level (KubeAI)
- Wave `2`: Dependent services (Harbor)
