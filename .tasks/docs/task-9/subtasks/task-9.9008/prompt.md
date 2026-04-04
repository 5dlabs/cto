Implement subtask 9008: Integrate push notifications for Morgan chat via Expo Notifications

## Objective
Set up Expo Notifications to receive push notifications for incoming Morgan messages when the app is backgrounded, including permission handling and notification tap navigation.

## Steps
1. Install `expo-notifications` and `expo-device`.
2. Create `lib/notifications/setup.ts`: request notification permissions on first launch (iOS requires explicit prompt). Store Expo push token and register it with the backend API.
3. Configure notification handler: when app is in foreground, show in-app notification banner (not system notification). When app is backgrounded, system notification appears.
4. Implement notification tap handler: tapping a Morgan message notification deep-links to the Chat tab and scrolls to the relevant message.
5. Configure `app.json` / `app.config.ts` with notification settings: icon, color, sound for Android; category for iOS.
6. Create `lib/notifications/tokenRefresh.ts`: handle push token refresh and re-register with backend.
7. Handle permission denied gracefully: show settings prompt if user previously denied.

## Validation
Mock Expo Notifications module. Verify push token is obtained and sent to backend API on first launch. Verify foreground notification renders in-app banner. Verify notification tap handler navigates to Chat tab. Verify permission denied state shows appropriate UI prompt.