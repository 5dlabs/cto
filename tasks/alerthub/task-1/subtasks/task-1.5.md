# Subtask 1.5: Deploy RabbitMQ Cluster

## Parent Task
Task 1

## Agent
rabbitmq-deployer

## Parallelizable
Yes

## Description
Deploy RabbitMQ cluster for message queuing with high availability and policy management.

## Details
- Install RabbitMQ Cluster Operator
- Create RabbitMQ cluster with appropriate quorum
- Configure queues with mirroring policies
- Set up exchanges and bindings
- Implement dead letter queue handling
- Configure monitoring for queue depths

## Deliverables
- `rabbitmq-operator.yaml` - Cluster operator
- `rabbitmq-cluster.yaml` - RabbitMQ cluster CR
- `rabbitmq-queues.yaml` - Queue definitions
- `rabbitmq-policies.yaml` - Policy configurations

## Acceptance Criteria
- [ ] RabbitMQ operator is Running
- [ ] RabbitMQ pods are Running
- [ ] Management UI is accessible
- [ ] Queues can be declared and used
- [ ] Message publishing/consuming works

## Testing Strategy
- Connect with rabbitmqadmin
- Create test queue with policy
- Publish test messages
- Consume and verify order
- Test queue mirroring failover
