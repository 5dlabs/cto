# Implementation Prompt for Task 3

## Context
You are implementing "Integration Service with Effect (Nova - Bun/Elysia)" for the AlertHub notification platform.

## PRD Reference
See `../../prd.md` for full requirements.

## Task Requirements
Create the notification delivery service using Bun/Elysia with Effect TypeScript for type-safe error handling and channel integrations (Slack, Discord, email, webhooks).

## Implementation Details
Build Elysia server with Effect services for each channel type. Implement SlackService, DiscordService, EmailService, and WebhookService using Effect.retry patterns. Add MongoDB integration with Drizzle ORM, Kafka consumer with Effect Stream, and RabbitMQ task queue processing.

## Dependencies
This task depends on: task-1. Ensure those are complete before starting.

## Testing Requirements
All integration endpoints respond correctly, Effect services handle errors gracefully, delivery retry logic works with exponential backoff, Kafka events are consumed successfully, and test deliveries reach external channels

## Decision Points to Address

The following decisions need to be made during implementation:

### d5: Retry strategy for failed deliveries
**Category**: error-handling | **Constraint**: soft

Options:
1. exponential backoff with 3 retries
2. linear backoff with 5 retries
3. configurable retry policy per channel

Document your choice and rationale in the implementation.

### d6: OAuth2 token storage and refresh mechanism
**Category**: api-design | **Constraint**: open

Options:
1. store in MongoDB with Effect.cached
2. external token service
3. Redis-based token cache

Document your choice and rationale in the implementation.


## Deliverables
1. Source code implementing the requirements
2. Unit tests with >80% coverage
3. Integration tests for external interfaces
4. Documentation updates as needed
5. Decision point resolutions documented

## Notes
- Follow project coding standards
- Use Effect TypeScript patterns where applicable
- Ensure proper error handling and logging
