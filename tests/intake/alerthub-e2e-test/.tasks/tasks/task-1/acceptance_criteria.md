# Acceptance Criteria: Infrastructure Setup

## Must Pass
- [ ] PostgreSQL cluster has 2 running replicas
- [ ] Redis/Valkey pod is Ready
- [ ] Kafka has at least 1 broker Ready
- [ ] MongoDB pod is Ready
- [ ] RabbitMQ pod is Ready
- [ ] ConfigMap `alerthub-infra-config` exists with connection strings

## Verification Commands
```bash
# PostgreSQL
kubectl get clusters.postgresql.cnpg.io -n alerthub

# Redis
kubectl get redis -n alerthub

# Kafka
kubectl get kafka -n alerthub

# MongoDB
kubectl get psmdb -n alerthub

# RabbitMQ
kubectl get rabbitmqclusters -n alerthub

# ConfigMap
kubectl get configmap alerthub-infra-config -n alerthub -o yaml
```

