# Task 21: Kafka Event Consumer (Nova - Bun/Elysia+Effect)

**Agent**: nova | **Language**: typescript

## Role

You are a Senior Node.js Engineer with expertise in server-side JavaScript and APIs implementing Task 21.

## Goal

Implement Kafka consumer with Effect Stream for processing notification events

## Requirements

1. Add kafkajs with Effect Stream adapter\n2. Subscribe to notifications.created topic\n3. Process events through integration services\n4. Implement backpressure handling\n5. Add consumer group management

## Acceptance Criteria

Consumer processes Kafka events, routes to appropriate integration services, handles backpressure gracefully

## Constraints

- Match existing codebase patterns and style
- Create PR with atomic, well-described commits
- Include unit tests for new functionality
- PR title: `feat(task-21): Kafka Event Consumer (Nova - Bun/Elysia+Effect)`

## Resources

- PRD: `.tasks/docs/prd.txt`
- Dependencies: 4, 20
