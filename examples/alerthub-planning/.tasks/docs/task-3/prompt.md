# Task 3: Build Integration Service with Effect (Nova - Bun/Elysia)

**Agent**: nova | **Language**: typescript

## Role

You are a TypeScript Engineer specializing in Effect and Bun/Elysia implementing Task 3.

## Goal

Create the integration service using Bun, Elysia, and Effect TypeScript to handle delivery to external channels (Slack, Discord, email, webhooks). Uses Effect for type-safe error handling and composable services.

## Requirements

1. Initialize Bun project with Elysia framework
2. Set up Effect services architecture (SlackService, DiscordService, EmailService)
3. Implement Effect Schema for request/response validation
4. Create MongoDB connection with Drizzle ORM
5. Build integration CRUD endpoints with Effect error handling
6. Implement Kafka consumer with Effect Stream
7. Add RabbitMQ task queue processing
8. Build delivery services with Effect.retry and exponential backoff
9. Add template rendering with Effect error handling
10. Implement OAuth2 token refresh with Effect.cached

## Acceptance Criteria

Service starts with Bun, Effect services initialize correctly, integration CRUD operations work, Kafka messages are consumed, deliveries succeed to test channels (Slack webhook, email), retry logic works on failures, and templates render correctly.

## Constraints

- Match existing codebase patterns
- Create PR with atomic commits
- Include unit tests
- PR title: `feat(task-3): Build Integration Service with Effect (Nova - Bun/Elysia)`

## Decision Points

### d5: Should integration configs be stored as flexible JSON or strongly typed schemas?
**Category**: api-design | **Constraint**: open

Options:
1. flexible-json
2. typed-schemas
3. hybrid-approach

### d6: How many retry attempts should be made for failed deliveries?
**Category**: error-handling | **Constraint**: soft

Options:
1. 3-attempts
2. 5-attempts
3. configurable-per-channel


## Resources

- PRD: `.tasks/docs/prd.md`
- Dependencies: task-1
