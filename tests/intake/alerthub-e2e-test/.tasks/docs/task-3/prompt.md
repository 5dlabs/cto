# Task 3: Implement Integration Service (Nova - Bun/Elysia + Effect)

**Agent**: nova | **Language**: typescript

## Role

You are a Senior Node.js Engineer with expertise in server-side JavaScript and APIs implementing Task 3.

## Goal

Build the channel delivery service with Effect for type-safe error handling, composable services, and robust retry logic

## Requirements

1. Initialize Bun project with Elysia and Effect:
   bun init integration-service
   bun add elysia@1 effect@3 @effect/platform @effect/schema drizzle-orm mongodb kafkajs amqplib handlebars

2. Setup Effect services architecture:
   - Define service interfaces using Context.Tag
   - SlackService: deliver(integration, message) => Effect<DeliveryResult, SlackDeliveryError>
   - DiscordService: deliver(integration, message) => Effect<DeliveryResult, DiscordDeliveryError>
   - EmailService: deliver(integration, message) => Effect<DeliveryResult, EmailDeliveryError>
   - WebhookService: deliver(integration, message) => Effect<DeliveryResult, WebhookDeliveryError>
   - PushService: deliver(integration, message) => Effect<DeliveryResult, PushDeliveryError>

3. Define Effect Schemas for validation:
   const Channel = Schema.Literal("slack", "discord", "email", "push", "webhook")
   const Integration = Schema.Struct({ id: Schema.String, tenantId: Schema.String, channel: Channel, name: Schema.String, config: Schema.Union(...), enabled: Schema.Boolean })
   const SlackConfig = Schema.Struct({ webhookUrl: Schema.String, channel: Schema.optional(Schema.String) })
   const DiscordConfig = Schema.Struct({ webhookUrl: Schema.String })
   const EmailConfig = Schema.Struct({ smtpHost: Schema.String, smtpPort: Schema.Number, fromAddress: Schema.String })
   const WebhookConfig = Schema.Struct({ url: Schema.String, headers: Schema.optional(Schema.Record({ key: Schema.String, value: Schema.String })), secret: Schema.optional(Schema.String) })

4. Implement service layers with Effect.Layer:
   const SlackServiceLive = Layer.succeed(SlackService, SlackService.of({
     deliver: (integration, message) => HttpClient.request.post(integration.config.webhookUrl).pipe(
       HttpClient.request.jsonBody(message),
       HttpClient.client.fetchOk,
       Effect.retry(Schedule.exponential("1 second").pipe(Schedule.compose(Schedule.recurs(3)))),
       Effect.mapError((e) => new SlackDeliveryError({ message: e.message }))
     )
   }))

5. Build Elysia API:
   POST /api/v1/integrations - Create integration (validate with Effect Schema)
   GET /api/v1/integrations - List integrations (query MongoDB with Drizzle)
   GET /api/v1/integrations/:id - Get integration
   PATCH /api/v1/integrations/:id - Update integration
   DELETE /api/v1/integrations/:id - Delete integration
   POST /api/v1/integrations/:id/test - Test connectivity (call service.deliver)
   POST /api/v1/integrations/:id/deliver - Manual delivery trigger

6. Implement Kafka consumer with Effect Stream:
   - Connect to Kafka bootstrap servers
   - Subscribe to alerthub.notifications.created topic
   - Transform Kafka messages to Effect Stream
   - Process with Effect.flatMap to route to appropriate service
   - Handle backpressure with Stream.buffer

7. Implement RabbitMQ consumer with Effect Stream:
   - Connect to RabbitMQ
   - Consume from integration.*.delivery queues
   - Route messages to channel services
   - Acknowledge on success, nack on failure (requeue with limit)

8. Implement template rendering:
   - Store templates in MongoDB
   - Use Handlebars for rendering
   - Wrap in Effect for error handling

9. Add rate limiting per channel:
   - Use Effect.Semaphore for concurrency control
   - Configure per-channel limits (Slack: 1 req/sec, Discord: 5 req/sec, etc.)

10. Create Dockerfile:
   FROM oven/bun:1.1
   WORKDIR /app
   COPY package.json bun.lockb ./
   RUN bun install --frozen-lockfile
   COPY . .
   CMD ["bun", "run", "src/index.ts"]

## Acceptance Criteria

1. Unit tests for Effect services with Effect.runPromise
2. Test retry logic with mock failures
3. Test Effect Schema validation with invalid inputs
4. Integration tests with mock Slack/Discord/Email APIs
5. Test Kafka consumer with test messages
6. Test RabbitMQ consumer with test queues
7. Verify template rendering with various data
8. Test rate limiting with Effect.Semaphore
9. Test error handling and tagged error types
10. Load test delivery throughput

## Constraints

- Match existing codebase patterns and style
- Create PR with atomic, well-described commits
- Include unit tests for new functionality
- PR title: `feat(task-3): Implement Integration Service (Nova - Bun/Elysia + Effect)`

## Resources

- PRD: `.tasks/docs/prd.txt`
- Dependencies: 1
