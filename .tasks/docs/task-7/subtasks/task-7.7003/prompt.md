Implement subtask 7003: Implement login page with credentials form

## Objective
Build the `/login` page with a username/password form using shadcn/ui Input and Button components. The form submits credentials to `POST /api/auth/login` (proxied through a Next.js API route to avoid CORS). On success, the API route sets an httpOnly cookie containing the JWT. On failure, display an inline error message.

## Steps
1. Create `src/app/login/page.tsx` with a centered card layout (shadcn/ui Card).
2. Add a form with username (Input) and password (Input type='password') fields, plus a Submit button.
3. Use React `useState` for form state and `useTransition` or `useFormStatus` for loading state.
4. Create `src/app/api/auth/login/route.ts` — a Next.js API route that:
   a. Receives `{ username, password }` from the form POST.
   b. Forwards the request to `${PM_SERVER_URL}/api/auth/login`.
   c. On success, sets an `httpOnly`, `secure`, `sameSite: 'strict'` cookie named `auth_token` with the JWT from the response.
   d. Returns `{ success: true }` to the client.
   e. On failure, returns `{ success: false, error: 'Invalid credentials' }` with 401 status.
5. On successful login, redirect to `/dashboard` using `next/navigation` `useRouter().push()`.
6. Display validation errors using shadcn/ui Alert (destructive variant) below the form.

## Validation
Login page renders with username and password fields. Submitting valid credentials results in a redirect to `/dashboard` and an httpOnly cookie is set (verify with browser dev tools or response headers). Submitting invalid credentials shows an inline error Alert. The JWT is NOT stored in localStorage.