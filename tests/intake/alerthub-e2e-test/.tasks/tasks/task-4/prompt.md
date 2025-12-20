# Task 4: Integration Service (Bun/Elysia + Effect)

## Agent: Nova
## Priority: High
## Runtime: Bun 1.1+
## Framework: Elysia with Effect TypeScript

## Objective
Build a channel delivery service for external integrations using Effect for type-safe error handling.

## Endpoints
- `POST /api/v1/integrations` - Create new integration
- `GET /api/v1/integrations` - List integrations for tenant
- `GET /api/v1/integrations/:id` - Get integration details
- `PATCH /api/v1/integrations/:id` - Update integration
- `DELETE /api/v1/integrations/:id` - Delete integration
- `POST /api/v1/integrations/:id/test` - Test integration connectivity

## Supported Channels
- Slack: Incoming webhooks, Bot API
- Discord: Webhooks
- Email: SMTP (SendGrid, AWS SES)
- Push: FCM for mobile
- Webhook: Custom HTTP endpoints

## Dependencies
- MongoDB: Integration configs, templates
- RabbitMQ: Task queue for delivery jobs
- Kafka: Consume notification events

## Acceptance Criteria
- [ ] All endpoints work correctly
- [ ] Effect error handling implemented
- [ ] Tests pass with `bun test`
- [ ] ESLint passes

