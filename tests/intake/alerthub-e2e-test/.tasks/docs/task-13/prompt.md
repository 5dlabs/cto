# Task 13: Kafka Event Publishing (Rex - Rust/Axum)

**Agent**: rex | **Language**: rust

## Role

You are a Senior Rust Engineer with expertise in systems programming and APIs implementing Task 13.

## Goal

Integrate Kafka producer to publish notification events for downstream processing

## Requirements

1. Add rdkafka crate for Kafka integration\n2. Configure Kafka producer with proper serialization\n3. Publish events to notifications.created topic\n4. Handle Kafka connection failures gracefully\n5. Add producer metrics

## Acceptance Criteria

Notification submissions trigger Kafka events, events visible in Kafka topic, service handles Kafka downtime gracefully

## Constraints

- Match existing codebase patterns and style
- Create PR with atomic, well-described commits
- Include unit tests for new functionality
- PR title: `feat(task-13): Kafka Event Publishing (Rex - Rust/Axum)`

## Resources

- PRD: `.tasks/docs/prd.txt`
- Dependencies: 4, 12
