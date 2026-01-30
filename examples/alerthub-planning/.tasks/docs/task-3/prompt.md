# Task 3: Integration Service with Effect (Nova - Bun/Elysia)

**Agent**: nova | **Language**: typescript

## Role

You are a TypeScript Engineer specializing in Effect and Bun/Elysia implementing Task 3.

## Goal

Create the notification delivery service using Bun/Elysia with Effect TypeScript for type-safe error handling and channel integrations (Slack, Discord, email, webhooks).

## Requirements

Build Elysia server with Effect services for each channel type. Implement SlackService, DiscordService, EmailService, and WebhookService using Effect.retry patterns. Add MongoDB integration with Drizzle ORM, Kafka consumer with Effect Stream, and RabbitMQ task queue processing.

## Acceptance Criteria

All integration endpoints respond correctly, Effect services handle errors gracefully, delivery retry logic works with exponential backoff, Kafka events are consumed successfully, and test deliveries reach external channels

## Constraints

- Match existing codebase patterns and style
- Create PR with atomic, well-described commits
- Include unit tests for new functionality
- PR title: `feat(task-3): Integration Service with Effect (Nova - Bun/Elysia)`

## Decision Points

### d5: Retry strategy for failed deliveries
**Category**: error-handling | **Constraint**: soft

Options:
1. exponential backoff with 3 retries
2. linear backoff with 5 retries
3. configurable retry policy per channel

### d6: OAuth2 token storage and refresh mechanism
**Category**: api-design | **Constraint**: open

Options:
1. store in MongoDB with Effect.cached
2. external token service
3. Redis-based token cache


## Resources

- PRD: `.tasks/docs/prd.md`
- Dependencies: task-1
