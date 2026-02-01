# Task 6: Develop Mobile Application (Tap - Expo/React Native)

**Agent**: tap | **Language**: tsx

## Role

You are a Mobile Engineer specializing in React Native and Expo implementing Task 6.

## Goal

Build the mobile app using Expo SDK 50+ and React Native with push notification support, offline caching, and biometric authentication for iOS and Android.

## Requirements

1. Initialize Expo project with React Native and NativeWind
2. Set up navigation with bottom tabs and stack navigation
3. Implement push notification registration (FCM/APNs)
4. Build notification feed screen with pull-to-refresh
5. Create notification detail screen with actions
6. Add biometric authentication (Face ID, fingerprint)
7. Implement offline notification caching with AsyncStorage
8. Build integrations and settings screens
9. Add deep linking for notification details
10. Configure app badge count and background processing

## Acceptance Criteria

App builds for both iOS and Android, push notifications are received and displayed, biometric authentication works, offline cached notifications are available, navigation flows correctly, deep links open specific notifications, and app badge shows unread count.

## Constraints

- Match existing codebase patterns
- Create PR with atomic commits
- Include unit tests
- PR title: `feat(task-6): Develop Mobile Application (Tap - Expo/React Native)`

## Decision Points

### d11: How should push notifications be grouped when multiple arrive quickly?
**Category**: ux-behavior | **Constraint**: soft | ⚠️ **Requires Approval**

Options:
1. individual-notifications
2. grouped-by-channel
3. summary-notification

### d12: How many notifications should be cached offline?
**Category**: performance | **Constraint**: soft

Options:
1. last-100
2. last-500
3. configurable-limit


## Resources

- PRD: `.tasks/docs/prd.md`
- Dependencies: task-2, task-3, task-4
