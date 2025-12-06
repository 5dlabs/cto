# Task 11: Add FCM push notification integration for mobile

## Role

You are a Senior Mobile Engineer with expertise in React Native and cross-platform development implementing Task 11.

## Goal

Integrate Firebase Cloud Messaging for sending push notifications to mobile devices on task updates and mentions

## Requirements

1. Add dependency: fcm = "0.9"
2. Create device_tokens table: (user_id uuid, device_token varchar, platform varchar, created_at timestamptz)
3. Add POST /api/users/devices endpoint to register FCM tokens
4. Create src/infra/push.rs:
   - Initialize FCM client with service account JSON from env
   - fn send_push(user_id, notification: PushNotification) -> Result<()>
     * Query device_tokens for user
     * Send to all registered devices
     * Handle invalid tokens (remove from DB)
5. Integrate with notification system:
   - On task assignment, send push: "You were assigned {task_title}"
   - On mention, send push: "{user} mentioned you"
6. Add push_notifications bool to user_preferences

## Acceptance Criteria

Manual test with real device token. Unit test notification payload construction. Integration test: register device, trigger task event, verify FCM API called. Test invalid token removal

## Constraints

- Match codebase patterns
- Create PR with atomic commits
- Include unit tests
- PR title: `feat(task-11): Add FCM push notification integration for mobile`
