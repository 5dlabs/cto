# Task 26: Implement FCM push notifications for mobile

## Role

You are a Senior Mobile Engineer with expertise in React Native and cross-platform development implementing Task 26.

## Goal

Integrate Firebase Cloud Messaging for mobile push notifications with device token management and notification delivery.

## Requirements

1. Add dependency: fcm = "0.9"
2. Create migrations/006_device_tokens.sql:
   - id (UUID PK), user_id (FK users), token (UNIQUE), platform (ENUM: ios, android), created_at, last_used_at
3. Implement infra/push.rs:
   - FcmService with API key from config
   - send_push_notification(user_id, title, body, data) -> Result<()>
   - Fetch active device tokens for user (last_used_at within 30 days)
   - Handle invalid token responses, delete from DB
4. Create api/devices.rs:
   - POST /api/devices/register (requires auth) -> stores FCM token
   - DELETE /api/devices/:token (requires auth) -> removes token
5. Integrate with task events:
   - Send push on task assignment
   - Send push on @mention
   - Respect user notification preferences
6. Add FCM_API_KEY to config
7. Include deep link data in notification payload: {task_id, team_id}

## Acceptance Criteria

Integration tests with FCM sandbox. Register device token, create task assigned to user, verify push sent. Test invalid token cleanup. Verify preferences disable push. Test multiple devices per user. Manual testing on iOS/Android devices with deep links.

## Constraints

- Match codebase patterns
- Create PR with atomic commits
- Include unit tests
- PR title: `feat(task-26): Implement FCM push notifications for mobile`
