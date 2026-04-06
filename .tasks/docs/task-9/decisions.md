## Decision Points

- The details list both a WebView approach (EXPO_PUBLIC_MORGAN_CHAT_URL) and a native EventSource/WebSocket approach as the preferred option. The final choice determines the chat screen architecture, testability, and whether Morgan must expose a native-compatible API endpoint.
- The details suggest react-native-calendars 'or similar' for the product detail DateRangePicker. Library choice affects bundle size, iOS/Android parity, and NativeWind 4 styling compatibility.
- The details offer two layout options for the portfolio tab. MasonryFlashList provides true masonry layout but is a separate package with different stability characteristics than FlashList.

## Coordination Notes

- Agent owner: tap
- Primary stack: Expo/React Native/NativeWind 4/Effect 3.x