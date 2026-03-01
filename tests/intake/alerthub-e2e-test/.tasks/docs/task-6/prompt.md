# Task 6: RabbitMQ Task Queue Setup (Bolt - Kubernetes)

**Agent**: bolt | **Language**: yaml

## Role

You are a Senior DevOps Engineer with expertise in Kubernetes, GitOps, and CI/CD implementing Task 6.

## Goal

Deploy RabbitMQ operator for integration delivery task queues

## Requirements

1. Install RabbitMQ operator\n2. Create RabbitMQ cluster with 1 instance\n3. Configure queues for each integration type\n4. Set up dead letter queues\n5. Configure management UI access

## Acceptance Criteria

RabbitMQ is running, management UI accessible, can publish/consume messages, DLQs are configured

## Constraints

- Match existing codebase patterns and style
- Create PR with atomic, well-described commits
- Include unit tests for new functionality
- PR title: `feat(task-6): RabbitMQ Task Queue Setup (Bolt - Kubernetes)`

## Resources

- PRD: `.tasks/docs/prd.txt`
- Dependencies: 1
