# Task 18: Implement Integration Service (Nova - Bun/Elysia + Effect)

**Agent**: nova | **Language**: typescript

## Role

You are a Senior Node.js Engineer with expertise in server-side JavaScript and APIs implementing Task 18.

## Goal

Build the channel delivery service using Bun, Elysia, and Effect TypeScript. Handles integration management and notification delivery to Slack, Discord, Email, Push, and Webhooks with type-safe error handling, retry logic, and rate limiting using Effect's composable patterns.

## Requirements

1. Initialize Bun project:

```bash
bun init
bun add elysia effect @effect/platform @effect/schema drizzle-orm mongodb kafkajs amqplib
bun add -d @types/node drizzle-kit
```

2. Setup Effect Schema for data models (src/schemas.ts):

```typescript
import { Schema } from "effect"

export const Channel = Schema.Literal("slack", "discord", "email", "push", "webhook")
export type Channel = Schema.Schema.Type<typeof Channel>

export const SlackConfig = Schema.Struct({
  webhookUrl: Schema.String.pipe(Schema.pattern(/^https:\/\/hooks\.slack\.com/)),
  channel: Schema.optional(Schema.String),
})

export const DiscordConfig = Schema.Struct({
  webhookUrl: Schema.String.pipe(Schema.pattern(/^https:\/\/discord\.com\/api\/webhooks/)),
})

export const EmailConfig = Schema.Struct({
  smtpHost: Schema.String,
  smtpPort: Schema.Number.pipe(Schema.int(), Schema.between(1, 65535)),
  fromAddress: Schema.String.pipe(Schema.pattern(/^[^@]+@[^@]+$/)),
  username: Schema.String,
  password: Schema.String,
})

export const WebhookConfig = Schema.Struct({
  url: Schema.String.pipe(Schema.pattern(/^https?:\/\//)),
  headers: Schema.optional(Schema.Record({ key: Schema.String, value: Schema.String })),
  secret: Schema.optional(Schema.String),
})

export const IntegrationConfig = Schema.Union(SlackConfig, DiscordConfig, EmailConfig, WebhookConfig)

export const Integration = Schema.Struct({
  id: Schema.String,
  tenantId: Schema.String,
  channel: Channel,
  name: Schema.String.pipe(Schema.minLength(1), Schema.maxLength(100)),
  config: IntegrationConfig,
  enabled: Schema.Boolean,
  createdAt: Schema.Date,
  updatedAt: Schema.Date,
})
export type Integration = Schema.Schema.Type<typeof Integration>

export const NotificationPayload = Schema.Struct({
  title: Schema.String,
  body: Schema.String,
  metadata: Schema.optional(Schema.Record({ key: Schema.String, value: Schema.Unknown })),
})

export const DeliveryResult = Schema.Struct({
  success: Schema.Boolean,
  deliveredAt: Schema.optional(Schema.Date),
  error: Schema.optional(Schema.String),
})

// Error schemas as tagged unions
export class SlackDeliveryError extends Schema.TaggedError<SlackDeliveryError>("SlackDeliveryError")({
  message: Schema.String,
  statusCode: Schema.optional(Schema.Number),
}) {}

export class RateLimitError extends Schema.TaggedError<RateLimitError>("RateLimitError")({
  retryAfter: Schema.Number,
}) {}

export class ValidationError extends Schema.TaggedError<ValidationError>("ValidationError")({
  message: Schema.String,
  field: Schema.optional(Schema.String),
}) {}
```

3. Setup Drizzle ORM for MongoDB (src/db/schema.ts):

```typescript
import { ObjectId } from "mongodb"
import { pgTable, text, timestamp, boolean, jsonb } from "drizzle-orm/pg-core"

export const integrations = pgTable("integrations", {
  id: text("id").primaryKey().$defaultFn(() => new ObjectId().toString()),
  tenantId: text("tenant_id").notNull(),
  channel: text("channel").notNull(),
  name: text("name").notNull(),
  config: jsonb("config").notNull(),
  enabled: boolean("enabled").default(true),
  createdAt: timestamp("created_at").defaultNow(),
  updatedAt: timestamp("updated_at").defaultNow(),
})
```

4. Implement Effect Services for each channel (src/services/):

SlackService (src/services/slack.ts):

```typescript
import { Context, Effect, Layer, Schedule } from "effect"
import { HttpClient } from "@effect/platform"

export class SlackService extends Context.Tag("SlackService")<
  SlackService,
  {
    deliver: (integration: Integration, payload: NotificationPayload) => Effect.Effect<DeliveryResult, SlackDeliveryError>
  }
>() {}

export const SlackServiceLive = Layer.succeed(
  SlackService,
  SlackService.of({
    deliver: (integration, payload) =>
      Effect.gen(function* () {
        const config = integration.config as Schema.Schema.Type<typeof SlackConfig>
        const slackMessage = {
          channel: config.channel,
          text: payload.title,
          blocks: [{ type: "section", text: { type: "mrkdwn", text: payload.body } }],
        }

        const response = yield* HttpClient.request.post(config.webhookUrl).pipe(
          HttpClient.request.jsonBody(slackMessage),
          HttpClient.client.fetchOk,
          Effect.retry(Schedule.exponential("1 second").pipe(Schedule.compose(Schedule.recurs(3)))),
          Effect.mapError((e) => new SlackDeliveryError({ message: e.message, statusCode: e.status }))
        )

        return { success: true, deliveredAt: new Date() }
      })
  })
)
```

DiscordService (src/services/discord.ts):

```typescript
export class DiscordService extends Context.Tag("DiscordService")<
  DiscordService,
  {
    deliver: (integration: Integration, payload: NotificationPayload) => Effect.Effect<DeliveryResult, SlackDeliveryError>
  }
>() {}

export const DiscordServiceLive = Layer.succeed(
  DiscordService,
  DiscordService.of({
    deliver: (integration, payload) =>
      Effect.gen(function* () {
        const config = integration.config as Schema.Schema.Type<typeof DiscordConfig>
        const discordMessage = {
          content: payload.title,
          embeds: [{ description: payload.body, color: 5814783 }],
        }

        yield* HttpClient.request.post(config.webhookUrl).pipe(
          HttpClient.request.jsonBody(discordMessage),
          HttpClient.client.fetchOk,
          Effect.retry(Schedule.exponential("1 second").pipe(Schedule.compose(Schedule.recurs(3)))),
          Effect.mapError((e) => new SlackDeliveryError({ message: e.message }))
        )

        return { success: true, deliveredAt: new Date() }
      })
  })
)
```

EmailService (src/services/email.ts):

```typescript
import * as nodemailer from "nodemailer"

export class EmailService extends Context.Tag("EmailService")<
  EmailService,
  {
    deliver: (integration: Integration, payload: NotificationPayload) => Effect.Effect<DeliveryResult, SlackDeliveryError>
  }
>() {}

export const EmailServiceLive = Layer.effect(
  EmailService,
  Effect.gen(function* () {
    return EmailService.of({
      deliver: (integration, payload) =>
        Effect.gen(function* () {
          const config = integration.config as Schema.Schema.Type<typeof EmailConfig>
          const transporter = nodemailer.createTransport({
            host: config.smtpHost,
            port: config.smtpPort,
            auth: { user: config.username, pass: config.password },
          })

          yield* Effect.tryPromise({
            try: () => transporter.sendMail({
              from: config.fromAddress,
              to: payload.metadata?.recipient || config.fromAddress,
              subject: payload.title,
              text: payload.body,
            }),
            catch: (e) => new SlackDeliveryError({ message: String(e) }),
          })

          return { success: true, deliveredAt: new Date() }
        })
    })
  })
)
```

WebhookService (src/services/webhook.ts):

```typescript
import * as crypto from "crypto"

export class WebhookService extends Context.Tag("WebhookService")<
  WebhookService,
  {
    deliver: (integration: Integration, payload: NotificationPayload) => Effect.Effect<DeliveryResult, SlackDeliveryError>
  }
>() {}

export const WebhookServiceLive = Layer.succeed(
  WebhookService,
  WebhookService.of({
    deliver: (integration, payload) =>
      Effect.gen(function* () {
        const config = integration.config as Schema.Schema.Type<typeof WebhookConfig>
        const body = JSON.stringify(payload)

        let headers = config.headers || {}
        if (config.secret) {
          const signature = crypto.createHmac("sha256", config.secret).update(body).digest("hex")
          headers["X-Webhook-Signature"] = signature
        }

        yield* HttpClient.request.post(config.url).pipe(
          HttpClient.request.bodyText(body),
          HttpClient.request.setHeaders(headers),
          HttpClient.client.fetchOk,
          Effect.retry(Schedule.exponential("1 second").pipe(Schedule.compose(Schedule.recurs(3)))),
          Effect.mapError((e) => new SlackDeliveryError({ message: e.message }))
        )

        return { success: true, deliveredAt: new Date() }
      })
  })
)
```

5. Implement Kafka consumer with Effect Stream (src/consumers/kafka.ts):

```typescript
import { Effect, Stream } from "effect"
import { Kafka } from "kafkajs"

export const createNotificationStream = Effect.gen(function* () {
  const kafka = new Kafka({ brokers: [process.env.KAFKA_BROKERS!] })
  const consumer = kafka.consumer({ groupId: "integration-service" })

  yield* Effect.promise(() => consumer.connect())
  yield* Effect.promise(() => consumer.subscribe({ topic: "notifications-events", fromBeginning: false }))

  return Stream.async<{ notification: Notification }, Error>((emit) => {
    consumer.run({
      eachMessage: async ({ message }) => {
        const notification = JSON.parse(message.value!.toString())
        emit.single({ notification })
      },
    })
    return Effect.promise(() => consumer.disconnect())
  })
})
```

6. Implement delivery orchestrator (src/services/delivery.ts):

```typescript
import { Effect, Context } from "effect"

export class DeliveryOrchestrator extends Context.Tag("DeliveryOrchestrator")<
  DeliveryOrchestrator,
  {
    processNotification: (notification: Notification) => Effect.Effect<DeliveryResult, SlackDeliveryError | ValidationError>
  }
>() {}

export const DeliveryOrchestratorLive = Layer.effect(
  DeliveryOrchestrator,
  Effect.gen(function* () {
    const slack = yield* SlackService
    const discord = yield* DiscordService
    const email = yield* EmailService
    const webhook = yield* WebhookService

    return DeliveryOrchestrator.of({
      processNotification: (notification) =>
        Effect.gen(function* () {
          // 1. Fetch integration from MongoDB
          const integration = yield* fetchIntegration(notification.tenantId, notification.channel)

          if (!integration.enabled) {
            return { success: false, error: "Integration disabled" }
          }

          // 2. Route to appropriate service
          const result = yield* Effect.matchTag(integration.channel, {
            slack: () => slack.deliver(integration, notification.payload),
            discord: () => discord.deliver(integration, notification.payload),
            email: () => email.deliver(integration, notification.payload),
            webhook: () => webhook.deliver(integration, notification.payload),
          })

          return result
        })
    })
  })
)
```

7. Implement Elysia routes (src/index.ts):

```typescript
import { Elysia } from "elysia"
import { Effect, Layer } from "effect"

const app = new Elysia()
  .post("/api/v1/integrations", async ({ body }) => {
    const program = Effect.gen(function* () {
      const validated = yield* Schema.decodeUnknown(Integration)(body)
      // Insert to MongoDB using Drizzle
      const result = yield* Effect.tryPromise({
        try: () => db.insert(integrations).values(validated).returning(),
        catch: (e) => new ValidationError({ message: String(e) }),
      })
      return result[0]
    })

    return Effect.runPromise(
      program.pipe(
        Effect.provide(Layer.mergeAll(SlackServiceLive, DiscordServiceLive, EmailServiceLive, WebhookServiceLive))
      )
    )
  })

  .get("/api/v1/integrations", async ({ query }) => {
    const program = Effect.gen(function* () {
      const results = yield* Effect.tryPromise({
        try: () => db.select().from(integrations).where(eq(integrations.tenantId, query.tenantId)),
        catch: (e) => new ValidationError({ message: String(e) }),
      })
      return results
    })

    return Effect.runPromise(program)
  })

  .post("/api/v1/integrations/:id/test", async ({ params, body }) => {
    const program = Effect.gen(function* () {
      const orchestrator = yield* DeliveryOrchestrator
      const result = yield* orchestrator.processNotification({
        id: "test",
        tenantId: body.tenantId,
        channel: body.channel,
        payload: { title: "Test Notification", body: "This is a test" },
      })
      return result
    })

    return Effect.runPromise(
      program.pipe(Effect.provide(DeliveryOrchestratorLive))
    )
  })

  .listen(3000)
```

8. Start Kafka consumer in background (src/workers/consumer.ts):

```typescript
const main = Effect.gen(function* () {
  const stream = yield* createNotificationStream
  const orchestrator = yield* DeliveryOrchestrator

  yield* stream.pipe(
    Stream.mapEffect(({ notification }) => orchestrator.processNotification(notification)),
    Stream.runDrain
  )
})

Effect.runPromise(
  main.pipe(Effect.provide(DeliveryOrchestratorLive))
)
```

9. Create Dockerfile:

```dockerfile
FROM oven/bun:1.1-alpine
WORKDIR /app
COPY package.json bun.lockb ./
RUN bun install --production
COPY src ./src
EXPOSE 3000
CMD ["bun", "run", "src/index.ts"]
```

10. Create Kubernetes manifests:

- Deployment with 2 replicas (API server)
- Deployment with 3 replicas (Kafka consumer workers)
- Service (ClusterIP) exposing port 3000
- ConfigMap with MongoDB, Kafka, RabbitMQ connection strings
- Secret with SMTP credentials

## Acceptance Criteria

1. Unit tests for Effect services:

```typescript
import { Effect } from "effect"
import { describe, it, expect } from "bun:test"

describe("SlackService", () => {
  it("should deliver notification successfully", async () => {
    const program = Effect.gen(function* () {
      const slack = yield* SlackService
      const result = yield* slack.deliver(mockIntegration, mockPayload)
      return result
    })

    const result = await Effect.runPromise(program.pipe(Effect.provide(SlackServiceLive)))
    expect(result.success).toBe(true)
  })
})
```

2. Test Effect Schema validation:

```typescript
const invalidIntegration = { channel: "invalid", ... }
const result = Schema.decodeUnknownEither(Integration)(invalidIntegration)
expect(Either.isLeft(result)).toBe(true)
```

3. Test retry logic with mock failures
4. Integration tests for Elysia routes:

```typescript
const response = await app.handle(new Request("http://localhost/api/v1/integrations", { method: "POST", body: JSON.stringify(validIntegration) }))
expect(response.status).toBe(201)
```

5. Test Kafka consumer with test messages
6. Test MongoDB CRUD operations with Drizzle
7. Test webhook signature verification
8. Test rate limiting with Effect Semaphore
9. Load test with 1000 concurrent deliveries
10. Deploy to Kubernetes and verify:
    - API pods respond to requests
    - Consumer pods process Kafka messages
    - Notifications are delivered to Slack/Discord
    - Failed deliveries are retried
11. End-to-end test: Submit notification via Router service → verify delivery via Integration service

## Constraints

- Match existing codebase patterns and style
- Create PR with atomic, well-described commits
- Include unit tests for new functionality
- PR title: `feat(task-18): Implement Integration Service (Nova - Bun/Elysia + Effect)`

## Resources

- PRD: `.tasks/docs/prd.txt`
- Dependencies: 16, 17
