Implement subtask 7004: Implement Next.js auth middleware and token refresh

## Objective
Create Next.js middleware that intercepts requests to protected routes (`/dashboard/*`) and verifies the JWT from the httpOnly cookie. If the token is expired or missing, redirect to `/login`. Implement a refresh mechanism via a Next.js API route that requests a new token before the current one expires.

## Steps
1. Create `src/middleware.ts` using Next.js Middleware API.
2. Configure the matcher to protect `/dashboard/:path*` routes.
3. In the middleware:
   a. Read the `auth_token` cookie from the request.
   b. Decode the JWT (without full verification — that's the server's job) to check the `exp` claim.
   c. If no token or token is expired, redirect to `/login` with a `?redirect=` query param preserving the original URL.
   d. If token is valid, allow the request through.
4. Create `src/app/api/auth/refresh/route.ts`:
   a. Read the current `auth_token` cookie.
   b. Forward it to `${PM_SERVER_URL}/api/auth/refresh` (or re-use login endpoint if refresh isn't supported).
   c. On success, set a new httpOnly cookie with the refreshed JWT.
   d. On failure, clear the cookie and return 401.
5. Create a client-side hook `src/hooks/useTokenRefresh.ts` that calls the refresh API route 1 minute before the 15-minute expiry, using the decoded `exp` claim to schedule the refresh.
6. Create `src/app/api/auth/logout/route.ts` that clears the cookie and add a logout button to the dashboard layout.

## Validation
Accessing `/dashboard` without a valid cookie redirects to `/login`. After login, `/dashboard` is accessible. After 15 minutes (or by manually expiring the cookie), the user is redirected to `/login`. The refresh mechanism extends the session when the hook fires before expiry. Logout clears the cookie and redirects to `/login`.