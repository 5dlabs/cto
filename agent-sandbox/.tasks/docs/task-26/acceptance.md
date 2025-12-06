# Acceptance Criteria: Task 26

- [ ] Integrate Firebase Cloud Messaging for mobile push notifications with device token management and notification delivery.
- [ ] Integration tests with FCM sandbox. Register device token, create task assigned to user, verify push sent. Test invalid token cleanup. Verify preferences disable push. Test multiple devices per user. Manual testing on iOS/Android devices with deep links.
- [ ] All requirements implemented
- [ ] Tests passing
- [ ] Code follows conventions
- [ ] PR created and ready for review

## Subtasks

- [ ] 26.1: Create device token database schema and migration
- [ ] 26.2: Implement FCM service with API key configuration
- [ ] 26.3: Create device registration and deletion API endpoints
- [ ] 26.4: Implement push notification sending with multi-device support
- [ ] 26.5: Implement invalid token cleanup based on FCM responses
- [ ] 26.6: Integrate push notifications with task events and user preferences
