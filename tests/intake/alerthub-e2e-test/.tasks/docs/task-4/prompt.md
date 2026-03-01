# Task 4: Kafka Event Streaming Setup (Bolt - Kubernetes)

**Agent**: bolt | **Language**: yaml

## Role

You are a Senior DevOps Engineer with expertise in Kubernetes, GitOps, and CI/CD implementing Task 4.

## Goal

Deploy Strimzi operator and Kafka cluster for async event processing

## Requirements

1. Install Strimzi operator\n2. Create Kafka cluster with 1 broker\n3. Create topics for notifications.created, notifications.delivered, notifications.failed\n4. Configure topic partitions and retention\n5. Set up Kafka Connect if needed

## Acceptance Criteria

Kafka cluster is running, topics exist with correct partitions, can produce/consume messages via kafka-console tools

## Constraints

- Match existing codebase patterns and style
- Create PR with atomic, well-described commits
- Include unit tests for new functionality
- PR title: `feat(task-4): Kafka Event Streaming Setup (Bolt - Kubernetes)`

## Resources

- PRD: `.tasks/docs/prd.txt`
- Dependencies: 1
