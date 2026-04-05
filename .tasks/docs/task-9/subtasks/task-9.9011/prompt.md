Implement subtask 9011: Configure EAS Build profiles, app branding, and production build pipeline

## Objective
Set up EAS Build with development, preview, and production profiles. Configure app icons, splash screen, and branding assets. Verify builds for both iOS and Android platforms.

## Steps
1. Install EAS CLI: `npm install -g eas-cli`. Run `eas init` to link project.
2. Create `eas.json` with three build profiles:
   - `development`: development client build with expo-dev-client, internal distribution.
   - `preview`: production-like build for testing, internal distribution (Ad Hoc for iOS, APK for Android).
   - `production`: store-ready build with proper signing.
3. Configure iOS provisioning: set up App Store Connect API key or Apple Developer credentials in EAS.
4. Configure Android keystore: generate upload keystore, store in EAS secrets.
5. Design and export app icon (1024x1024) matching Sigma-1 brand. Configure in `app.json` under `icon`.
6. Design and export splash screen with Sigma-1 logo. Configure `splash` in `app.json` with background color matching dark theme.
7. Configure adaptive icon for Android (foreground + background layers).
8. Set `app.json` metadata: `name`, `slug`, `version`, `ios.bundleIdentifier`, `android.package`, `ios.buildNumber`, `android.versionCode`.
9. Run `eas build --platform all --profile preview` to verify both platforms build successfully.
10. Configure OTA updates via `expo-updates` for non-native code changes.

## Validation
Run `eas build --platform ios --profile preview` and verify it completes without errors and produces a valid .ipa. Run `eas build --platform android --profile preview` and verify it produces a valid .apk. Install preview builds on physical devices and verify app icon, splash screen, and basic navigation work. Verify `eas.json` contains all three profiles with correct distribution settings.