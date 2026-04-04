## Decision Points

- NativeWind vs styled-components/Tamagui for the React Native styling approach — NativeWind mirrors web Tailwind classes but may have RN edge cases; Tamagui provides cross-platform tokens natively.
- AsyncStorage vs MMKV for local caching and offline storage — MMKV is significantly faster but adds a native dependency; AsyncStorage is built-in but slower for large datasets like equipment catalogs.
- Push notification service: Expo Push Notification service vs direct integration with FCM/APNs — Expo simplifies cross-platform but adds a proxy dependency.

## Coordination Notes

- Agent owner: tap
- Primary stack: Expo (React Native)