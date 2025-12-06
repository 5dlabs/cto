# Task 26: Implement FCM push notification integration

## Role

You are a Senior Mobile Engineer with expertise in React Native and cross-platform development implementing Task 26.

## Goal

Add Firebase Cloud Messaging support for mobile push notifications with device token management

## Requirements

1. Add dependency: fcm = "0.9"
2. Add device_tokens table: id, user_id, token, platform (ios/android), created_at
3. Create api/devices.rs:
   - POST /api/devices/register { token, platform } -> store device token for authenticated user
   - DELETE /api/devices/:id -> remove device token
4. Create infra/push.rs with:
   - struct PushService { fcm_client: fcm::Client }
   - async fn send_push(user_id: Uuid, notification: PushNotification) -> Result<()>
   - Query device_tokens for user, send to all devices
5. Integrate with task events: on task assignment/mention, send push if user enabled
6. Handle FCM errors: remove invalid tokens from database
7. Configure FCM via FIREBASE_SERVER_KEY env var

## Acceptance Criteria

Unit tests with mock FCM client, integration tests for device registration, verify push sent on task events, test invalid token cleanup

## Constraints

- Match codebase patterns
- Create PR with atomic commits
- Include unit tests
- PR title: `feat(task-26): Implement FCM push notification integration`
