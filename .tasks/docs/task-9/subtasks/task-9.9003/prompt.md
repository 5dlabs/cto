Implement subtask 9003: Build shared API client with JWT auth and environment configuration

## Objective
Create a shared API client module with configurable base URL per environment, JWT token storage in expo-secure-store, automatic token refresh, and Effect-based error handling.

## Steps
1. Install dependencies: `npx expo install expo-secure-store` and `npm install effect` (or `@effect/io` depending on version).
2. Create `lib/api/client.ts`: base HTTP client using `fetch` with interceptors for auth headers.
3. Implement environment config in `lib/config.ts`: read `EXPO_PUBLIC_API_BASE_URL` from env vars, support dev/staging/prod profiles.
4. Create `lib/auth/tokenStore.ts`: store JWT access token and refresh token in `expo-secure-store`. Export `getToken()`, `setTokens()`, `clearTokens()`.
5. Implement automatic `Authorization: Bearer <token>` header injection on every API request.
6. Implement token refresh logic: if 401 received, attempt refresh using stored refresh token, retry original request.
7. Create typed API modules: `lib/api/equipment.ts` (getCategories, getProducts, getProductDetail), `lib/api/quotes.ts` (createQuote, getQuotes), `lib/api/inventory.ts` (scanBarcode, checkIn, checkOut).
8. Wrap API calls in Effect for structured error handling (network errors, auth errors, validation errors).
9. Create a `useApi` hook or context provider that exposes the client instance to components.

## Validation
Unit test the API client: mock `fetch` to verify correct headers (Authorization, Content-Type) are sent. Test token refresh flow: mock a 401 response followed by successful refresh, verify retry succeeds. Test `expo-secure-store` read/write with mocked module. Verify environment config resolves correct base URL per EXPO_PUBLIC_API_BASE_URL value.