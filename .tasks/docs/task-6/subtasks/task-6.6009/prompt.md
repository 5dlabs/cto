Implement subtask 6009: Implement NATS integration for async publish pipeline with dead-letter handling

## Objective
Build the NATS subscriber that listens to the social.publish subject, dispatches to platform services, updates draft/published_posts records, and handles dead-letter after 3 failed attempts. Wire the approve and publish endpoints to emit NATS messages.

## Steps
1. Install `nats` npm package.
2. Create `src/services/NATSService.ts` as an Effect.Service:
   - Constructor connects to NATS server (NATS_URL from env).
   - Method `publish(subject: string, payload: object)`: Effect<void, NATSError> — JSON encode and publish.
   - Method `subscribe(subject: string, handler: (msg) => Effect<void, E>)`: Effect<Subscription, NATSError>.
   - Graceful shutdown: drain connection on SIGTERM.
3. Create `src/subscribers/publishSubscriber.ts`:
   - Subscribe to `social.publish` subject.
   - Message schema: { draft_id: string, platform: string, attempt: number }.
   - On message:
     a. Fetch draft from DB by draft_id.
     b. Call PlatformRouter.publishToPlatform(draft.platform, draft).
     c. On success: Update draft status to 'published', set published_at and platform_post_id. Insert published_posts record with post_url.
     d. On failure: If attempt < 3, republish to `social.publish` with attempt+1 and a delay header. If attempt >= 3, mark draft as 'failed' with error message, publish to `social.publish.dead` for observability.
   - Use Effect.catchAll for comprehensive error handling.
4. Wire NATS publish into draft endpoints (update 6004's routes):
   - `POST /drafts/:id/approve` → after DB update, publish `{ draft_id, platform, attempt: 1 }` to `social.publish`.
   - `POST /drafts/:id/publish` → same NATS publish.
5. Create `src/subscribers/index.ts` that starts all subscribers on app boot.
6. Create `NATSService.live.ts` — Effect Layer.

## Validation
Integration test with NATS test server (nats-server in Docker): (1) Publish message to social.publish, verify subscriber processes it, draft status updated to 'published', published_posts record created. (2) Mock platform service failure, verify message republished with attempt=2. (3) Mock 3 consecutive failures, verify draft marked 'failed' and message published to social.publish.dead. (4) Test approve endpoint publishes correct NATS message. (5) Test manual publish endpoint publishes correct NATS message. (6) Test graceful shutdown: start subscriber, send SIGTERM, verify drain completes.