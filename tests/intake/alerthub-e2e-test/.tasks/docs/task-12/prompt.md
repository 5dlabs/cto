# Task 12: Build and Publish Mobile App (Tap - Expo/React Native)

**Agent**: tap | **Language**: tsx

## Role

You are a Senior Mobile Engineer with expertise in React Native and cross-platform development implementing Task 12.

## Goal

Build production APK/IPA and prepare for app store submission

## Requirements

1. Configure EAS Build:
   - Install EAS CLI: npm install -g eas-cli
   - Login: eas login
   - Configure project: eas build:configure

2. Update app.json for production:
   - Set version and buildNumber
   - Configure app icons (1024x1024 for iOS, adaptive icon for Android)
   - Set splash screen
   - Configure deep linking scheme (alerthub://)
   - Set iOS bundle ID and Android package
   - Configure push notification credentials (FCM server key, APNs key)

3. Setup environment variables:
   - Create eas.json with production profile
   - Add API_URL, WS_URL environment variables
   - Configure secrets (API keys) in EAS

4. Build iOS app:
   - eas build --platform ios --profile production
   - Download IPA from EAS dashboard

5. Build Android app:
   - eas build --platform android --profile production
   - Download APK/AAB from EAS dashboard

6. Prepare App Store submission (iOS):
   - Create app in App Store Connect
   - Upload screenshots (6.5", 5.5" iPhone, 12.9" iPad)
   - Write app description, keywords, release notes
   - Set age rating, privacy policy URL
   - Upload IPA via Transporter or EAS Submit

7. Prepare Google Play submission (Android):
   - Create app in Google Play Console
   - Upload screenshots (phone, tablet, 7", 10")
   - Write app description, short description, release notes
   - Set content rating, privacy policy URL
   - Upload AAB via Google Play Console or EAS Submit

8. Configure push notifications:
   - Upload APNs key to EAS
   - Configure FCM server key in EAS
   - Test push notifications on TestFlight/internal testing

9. Submit for review:
   - iOS: Submit to App Store review
   - Android: Submit to Google Play review

## Acceptance Criteria

1. Test production build on physical devices
2. Verify push notifications work
3. Test deep linking
4. Test offline behavior
5. Test biometric authentication
6. Verify API calls to production backend
7. Test app lifecycle (background, foreground, killed)
8. Run automated tests on EAS
9. Beta test with TestFlight (iOS) and internal testing (Android)
10. Verify app store metadata (screenshots, description)

## Constraints

- Match existing codebase patterns and style
- Create PR with atomic, well-described commits
- Include unit tests for new functionality
- PR title: `feat(task-12): Build and Publish Mobile App (Tap - Expo/React Native)`

## Resources

- PRD: `.tasks/docs/prd.txt`
- Dependencies: 6, 8, 9, 10
