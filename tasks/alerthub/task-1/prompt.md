# Implementation Prompt: Task 1

**Agent:** Bolt
**Tech Stack:** Kubernetes, CloudNative-PG, Strimzi, Prometheus
**Status:** pending

## What to Build
Deploy foundational infrastructure including:
- PostgreSQL cluster
- MongoDB cluster  
- Redis/Valkey cache
- Kafka cluster
- RabbitMQ cluster
- Monitoring stack (Prometheus, Grafana, Loki)
- Kubernetes namespaces and network policies
- Ingress controller and load balancers
- Secrets management

## Implementation Details
Execute subtasks in order:
1. Deploy databases (PostgreSQL, MongoDB, Redis) - can run in parallel
2. Deploy messaging (Kafka, RabbitMQ) - can run in parallel
3. Configure namespaces and network policies
4. Deploy monitoring stack
5. Configure ingress and load balancers
6. Set up secrets management
7. Final review and validation

## Dependencies
None - this is the foundation task

## Testing Requirements
Verify all services are:
- Running and healthy
- Accessible via expected endpoints
- Persisting data correctly
- Backed up appropriately
- Monitored with dashboards
