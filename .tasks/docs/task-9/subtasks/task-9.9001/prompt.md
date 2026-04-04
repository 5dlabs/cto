Implement subtask 9001: Initialize Expo project with TypeScript and Expo Router tab navigation

## Objective
Scaffold the Expo SDK 51+ project with TypeScript configuration, install Expo Router, and configure file-based tab navigation with all five tab screens (Equipment, Quote, Chat, Scan, Profile).

## Steps
1. Run `npx create-expo-app sigma1-mobile --template expo-template-blank-typescript` or equivalent SDK 51+ template.
2. Install Expo Router: `npx expo install expo-router react-native-safe-area-context react-native-screens expo-linking expo-constants expo-status-bar`.
3. Configure `app.json` / `app.config.ts` with scheme, bundleIdentifier, and package name for Sigma-1.
4. Create the `app/` directory with `_layout.tsx` as root layout using `<Tabs>` from Expo Router.
5. Create five tab files: `app/(tabs)/equipment.tsx`, `app/(tabs)/quote.tsx`, `app/(tabs)/chat.tsx`, `app/(tabs)/scan.tsx`, `app/(tabs)/profile.tsx`.
6. Configure tab bar icons (use `@expo/vector-icons` or custom SVGs) and labels.
7. Add nested routes for Equipment tab: `app/(tabs)/equipment/index.tsx` (category list), `app/(tabs)/equipment/[categoryId].tsx` (product grid), `app/(tabs)/equipment/product/[productId].tsx` (product detail).
8. Verify TypeScript strict mode is enabled in `tsconfig.json`.
9. Configure path aliases (`@/components`, `@/lib`, `@/hooks`) in `tsconfig.json`.

## Validation
Run `npx expo start` and verify the app launches in iOS Simulator and Android Emulator with all five tabs visible. Tapping each tab renders its placeholder screen. Deep navigation in Equipment tab (index → category → product) works via Expo Router links.