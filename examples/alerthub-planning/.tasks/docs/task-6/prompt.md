# Task 6: Mobile Push Notification App (Tap - Expo)

**Agent**: tap | **Language**: tsx

## Role

You are a Mobile Engineer specializing in React Native and Expo implementing Task 6.

## Goal

Develop mobile application using Expo SDK 50+ for receiving push notifications and managing user preferences.

## Requirements

Create Expo app with screens for notification feed, detail view, integrations, settings, and profile. Implement FCM/APNs push registration, biometric authentication, offline caching, pull-to-refresh, deep linking, and NativeWind styling.

## Acceptance Criteria

App builds successfully, push notifications are received and displayed, biometric auth works on supported devices, offline notifications are cached and sync when online, deep links navigate to correct screens, and pull-to-refresh updates data

## Constraints

- Match existing codebase patterns and style
- Create PR with atomic, well-described commits
- Include unit tests for new functionality
- PR title: `feat(task-6): Mobile Push Notification App (Tap - Expo)`

## Decision Points

### d11: Push notification interaction behavior
**Category**: ux-behavior | **Constraint**: soft

Options:
1. tap opens notification detail
2. tap opens main feed
3. show action buttons on notification

### d12: Offline notification storage strategy
**Category**: data-model | **Constraint**: open

Options:
1. SQLite for full offline support
2. AsyncStorage for simple caching
3. Expo SecureStore for sensitive data only


## Resources

- PRD: `.tasks/docs/prd.md`
- Dependencies: task-3, task-4
