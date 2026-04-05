## Develop Mobile App (Tap - Expo/React Native)

### Objective
Build the Sigma-1 mobile app using Expo with equipment catalog browsing, self-service quote builder, dedicated Morgan chat screen, and barcode scanning for equipment check-in/check-out. Maintains visual consistency with the web frontend.

### Ownership
- Agent: tap
- Stack: Expo (React Native)
- Priority: medium
- Status: pending
- Dependencies: 2, 7

### Implementation Details
1. Initialize Expo project (SDK 51+) with TypeScript, Expo Router for file-based navigation.
2. Design system:
   - Port design tokens from web (Task 8) to React Native: colors, typography (using expo-font), spacing scale.
   - Use React Native equivalents of shadcn/ui patterns: NativeWind (TailwindCSS for RN) or styled components matching web aesthetic.
   - Component library: ProductCard, AvailabilityBadge, QuoteLineItem, ChatBubble, BarcodeScanner.
3. Navigation structure (Expo Router tabs):
   - **Equipment tab**: Category list → Product grid → Product detail with availability. Pull-to-refresh. Infinite scroll pagination.
   - **Quote tab**: Quote builder adapted for mobile. Step-by-step wizard with native date picker, equipment selector (searchable list), venue input, review screen. Submit creates opportunity.
   - **Chat tab** (dedicated screen per D10): Full-screen Morgan chat. WebSocket connection to Morgan agent. Message history. Support rich messages (product cards as interactive elements). Push notification integration for incoming messages.
   - **Scan tab**: Camera-based barcode scanner using `expo-camera` or `expo-barcode-scanner`. Scans equipment barcode → calls RMS ScanBarcode → shows equipment details and check-in/check-out actions.
   - **Profile/Settings tab**: User info, notification preferences, saved quotes.
4. API integration:
   - Shared API client module using `fetch` or `axios` with Effect for error handling.
   - Base URL configurable per environment (dev/staging/prod).
   - JWT token storage in `expo-secure-store`.
   - API key auth for service calls (if needed, else JWT from user session).
5. Morgan Chat (dedicated screen):
   - WebSocket connection to Morgan agent (same endpoint as web).
   - Push notifications via Expo Notifications for incoming Morgan messages when app is backgrounded.
   - Conversation persistence in AsyncStorage with sync to server.
   - Support photo sending (for social pipeline): use `expo-image-picker`, upload to social engine.
6. Barcode scanning:
   - `expo-camera` with barcode detection.
   - On scan: call RMS `POST /api/v1/inventory/scan` (ScanBarcode via REST gateway).
   - Display equipment details, current status, option to check-out or check-in.
   - Haptic feedback on successful scan.
7. Offline capability:
   - Cache equipment catalog locally using AsyncStorage or MMKV.
   - Queue quote submissions if offline, submit when connectivity restored.
8. Build configuration:
   - EAS Build profiles for development, preview, production.
   - iOS and Android builds.
   - App icons and splash screen matching Sigma-1 branding.

### Subtasks
- [ ] Initialize Expo project with TypeScript and Expo Router tab navigation: Scaffold the Expo SDK 51+ project with TypeScript configuration, install Expo Router, and configure file-based tab navigation with all five tab screens (Equipment, Quote, Chat, Scan, Profile).
- [ ] Port design system tokens and build shared component library: Port the web frontend's design tokens (colors, typography, spacing) to React Native using NativeWind or chosen styling approach. Build the shared component library: ProductCard, AvailabilityBadge, QuoteLineItem, ChatBubble.
- [ ] Build shared API client with JWT auth and environment configuration: Create a shared API client module with configurable base URL per environment, JWT token storage in expo-secure-store, automatic token refresh, and Effect-based error handling.
- [ ] Implement Equipment tab with category browsing, product grid, and infinite scroll: Build the Equipment tab screens: category list, product grid with infinite scroll pagination, product detail with availability display, and pull-to-refresh across list screens.
- [ ] Build Quote Builder tab with step-by-step wizard and submission: Implement the Quote tab as a multi-step wizard: equipment selection (searchable list), date range picker, venue input, review screen, and API submission to create an opportunity.
- [ ] Implement offline quote queuing and equipment catalog caching: Add offline capability: cache equipment catalog locally for offline browsing and queue quote submissions when offline, auto-submitting when connectivity is restored.
- [ ] Build Morgan Chat tab with WebSocket connection and message history: Implement the dedicated Morgan chat screen with full-screen WebSocket-based conversation, message history persistence in AsyncStorage, and support for rich messages (interactive product cards).
- [ ] Integrate push notifications for Morgan chat via Expo Notifications: Set up Expo Notifications to receive push notifications for incoming Morgan messages when the app is backgrounded, including permission handling and notification tap navigation.
- [ ] Implement Barcode Scan tab with camera scanner and RMS integration: Build the Scan tab with camera-based barcode detection using expo-camera, integration with RMS ScanBarcode API, equipment details display, and check-in/check-out actions with haptic feedback.
- [ ] Build Profile/Settings tab with user info and notification preferences: Implement the Profile tab displaying user information, notification preference toggles, saved quotes list, and sign-out functionality.
- [ ] Configure EAS Build profiles, app branding, and production build pipeline: Set up EAS Build with development, preview, and production profiles. Configure app icons, splash screen, and branding assets. Verify builds for both iOS and Android platforms.