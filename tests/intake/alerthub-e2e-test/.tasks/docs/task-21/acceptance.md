# Acceptance Criteria: Task 21

- [ ] Build the integration service that delivers notifications to external channels (Slack, Discord, email, webhooks) using Effect TypeScript for type-safe error handling, composable services, and robust retry logic. Consumes events from Kafka via Effect Stream and processes delivery tasks from RabbitMQ with backpressure management.
- [ ] 1. Unit tests for Effect services:
- Test SlackService.deliver with mock HttpClient
- Test EmailService.deliver with mock SMTP
- Test error handling and retry logic with Effect.retry
- Test Effect Schema validation for Integration configs

2. Integration tests with test containers:
- Spin up MongoDB, Kafka, RabbitMQ containers
- Test POST /api/v1/integrations creates integration in MongoDB
- Test GET /api/v1/integrations lists integrations
- Test POST /api/v1/integrations/:id/test delivers test message
- Test Kafka consumer processes notification events

3. Effect-specific tests:
- Test Effect.retry with simulated failures (should retry 3 times)
- Test Effect.Schema validation rejects invalid configs
- Test Effect Stream backpressure with high message volume
- Test Effect error propagation through service layers

4. End-to-end delivery tests:
- Mock Slack webhook endpoint, verify payload format
- Test rate limiting with Effect Semaphore
- Test concurrent deliveries to multiple channels
- Verify delivery events published to Kafka

5. Performance tests:
- Process 1,000 notifications/minute sustained
- Verify Effect Stream handles backpressure correctly
- Test retry logic doesn't cause memory leaks

6. Error handling tests:
- Test SlackDeliveryError caught and logged
- Test RateLimitError triggers backoff
- Test ConfigurationError fails immediately without retry
- [ ] All requirements implemented
- [ ] Tests passing
- [ ] Code follows conventions
- [ ] PR created and ready for review

## Subtasks

- [ ] 21.1: Initialize Bun project with Effect TypeScript dependencies and configuration
- [ ] 21.2: Define Effect Schema definitions for all domain models and error types
- [ ] 21.3: Implement Slack delivery service with Effect retry logic and error handling
- [ ] 21.4: Implement Discord delivery service with webhook integration
- [ ] 21.5: Implement Email delivery service with SMTP and template rendering
- [ ] 21.6: Implement Webhook delivery service with signature verification
- [ ] 21.7: Implement MongoDB repository layer with Effect error handling
- [ ] 21.8: Implement Kafka consumer with Effect Stream for notification processing
- [ ] 21.9: Implement RabbitMQ task queue integration for retry and dead-letter handling
- [ ] 21.10: Implement Elysia REST API routes for integration management
- [ ] 21.11: Implement main server with Effect Layer composition and consumer startup
- [ ] 21.12: Create Dockerfile, Kubernetes manifests, and deployment documentation
