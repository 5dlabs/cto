# Task 4: Integration Service (Nova - Bun/Elysia)

**Agent**: nova | **Language**: typescript

## Role

You are a Senior Node.js Engineer with expertise in server-side JavaScript and APIs implementing Task 4.

## Goal

Build Bun/Elysia service with Effect for channel delivery (Slack, Discord, email, push, webhooks)

## Requirements

Implement Effect Services for each channel with retry logic, Effect Schema validation, Kafka/RabbitMQ consumers, rate limiting via Effect Semaphore, OAuth2 token refresh, and webhook signature verification.

## Acceptance Criteria

Unit tests with Effect TestContext, integration tests with mock HTTP servers, E2E Kafka flow test

## Constraints

- Match existing codebase patterns and style
- Create PR with atomic, well-described commits
- Include unit tests for new functionality
- PR title: `feat(task-4): Integration Service (Nova - Bun/Elysia)`

## Resources

- PRD: `.tasks/docs/prd.txt`
- Dependencies: 1
