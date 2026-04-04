Implement subtask 9010: Build Profile/Settings tab with user info and notification preferences

## Objective
Implement the Profile tab displaying user information, notification preference toggles, saved quotes list, and sign-out functionality.

## Steps
1. Build `screens/profile/ProfileScreen.tsx`: Display user avatar, name, email, company from JWT claims or user API endpoint.
2. **Notification preferences**: Toggle switches for push notifications (Morgan messages, quote updates, equipment alerts). Persist preferences to backend API and locally.
3. **Saved quotes**: List of user's submitted and draft quotes. Each item shows quote ID, date, status (pending, approved, rejected), total. Tap navigates to quote detail.
4. **Sign out**: Clear JWT tokens from expo-secure-store, clear AsyncStorage caches, disconnect WebSocket, navigate to auth/login screen.
5. Add app version display at bottom of profile screen.
6. Optional: Theme toggle (if supporting light/dark modes), language selector.

## Validation
Render profile screen with mock user data, verify name/email displayed. Toggle notification preference, verify API call and local persistence. Verify saved quotes list renders mock quotes with correct status badges. Sign out test: verify tokens cleared from secure store, navigation to login screen triggered.