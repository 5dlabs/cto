# Task 6: Implement Mobile App (Tap - Expo/React Native)

**Agent**: tap | **Language**: tsx

## Role

You are a Senior Mobile Engineer with expertise in React Native and cross-platform development implementing Task 6.

## Goal

Build the mobile application for iOS and Android with push notification support and offline caching

## Requirements

1. Initialize Expo project:
   npx create-expo-app mobile-app --template blank-typescript
   cd mobile-app
   npx expo install expo-notifications expo-secure-store expo-router nativewind react-native-reanimated @tanstack/react-query

2. Setup NativeWind for styling:
   - Configure tailwind.config.js
   - Add NativeWind provider to app/_layout.tsx

3. Configure Expo Router file-based navigation:
   app/(tabs)/_layout.tsx - Bottom tab navigator
   app/(tabs)/index.tsx - Home screen (notifications feed)
   app/(tabs)/integrations.tsx - Integrations list
   app/(tabs)/settings.tsx - Settings screen
   app/(tabs)/profile.tsx - Profile screen
   app/notification/[id].tsx - Notification detail screen

4. Implement push notification registration:
   - Request permissions with expo-notifications
   - Register device token with FCM
   - Send token to backend API
   - Handle notification received (foreground/background)
   - Handle notification tapped (deep link to detail screen)

5. Build HomeScreen component:
   - Fetch notifications with TanStack Query
   - Display in FlatList with pull-to-refresh
   - Show notification card (title, body, timestamp, status badge)
   - Tap to navigate to detail screen
   - Swipe actions (mark as read, delete)

6. Build NotificationDetailScreen:
   - Display full notification with metadata
   - Show delivery status and events
   - Action buttons (retry, dismiss)
   - Share functionality

7. Build IntegrationsScreen:
   - List connected integrations
   - Show status indicator (enabled/disabled)
   - Tap to view integration details
   - No edit functionality (web-only)

8. Build SettingsScreen:
   - Notification preferences (enable/disable channels)
   - Quiet hours configuration
   - Biometric authentication toggle
   - Theme selection (light/dark/system)
   - Logout button

9. Build ProfileScreen:
   - Display user info (name, email, tenant)
   - Account statistics (total notifications, delivery rate)
   - App version and build number

10. Implement offline caching:
    - Use TanStack Query with persistQueryClient
    - Store notifications in AsyncStorage
    - Show cached data when offline
    - Sync on reconnection

11. Implement biometric authentication:
    - Use expo-local-authentication
    - Prompt for Face ID/fingerprint on app launch
    - Store auth state in expo-secure-store

12. Configure app.json:
    - Set app name, slug, version
    - Configure notification settings (icon, sound, badge)
    - Set iOS bundle ID and Android package
    - Configure deep linking scheme (alerthub://)

13. Build APK/IPA:
    - Configure EAS Build
    - Create build profiles (development, preview, production)
    - Run eas build --platform all

## Acceptance Criteria

1. Unit tests for components with Jest and React Native Testing Library
2. Test push notification handling (received, tapped)
3. Test offline behavior (cache, sync)
4. Test biometric authentication flow
5. Test deep linking to notification detail
6. Test pull-to-refresh and pagination
7. Manual testing on iOS simulator and Android emulator
8. Test on physical devices (iOS and Android)
9. Verify app badge count updates
10. Test app lifecycle (foreground, background, killed)

## Constraints

- Match existing codebase patterns and style
- Create PR with atomic, well-described commits
- Include unit tests for new functionality
- PR title: `feat(task-6): Implement Mobile App (Tap - Expo/React Native)`

## Resources

- PRD: `.tasks/docs/prd.txt`
- Dependencies: 2, 3, 4
