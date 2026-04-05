Implement subtask 6014: Implement RBAC middleware with JWT validation

## Objective
Create Elysia middleware that validates JWT tokens from incoming requests and checks user roles against the sigma1-rbac-roles ConfigMap for authorization on social media endpoints.

## Steps
1. Install `jose` (or `jsonwebtoken`) for JWT verification.
2. Create `src/middleware/rbac.ts`.
3. Implement `authMiddleware` as an Elysia derive/beforeHandle:
   - Extract Bearer token from Authorization header.
   - Verify JWT signature using the public key/secret from environment (JWT_SECRET or JWKS_URL).
   - Decode claims: `sub`, `role`, `exp`.
   - Reject expired tokens with 401.
   - Reject missing/invalid tokens with 401.
4. Implement `requireRole(...roles: string[])` guard:
   - Read the sigma1-rbac-roles ConfigMap (mounted as file or env var) to get role definitions.
   - Check if the user's role from JWT is in the allowed roles list.
   - Return 403 if role is insufficient.
5. Apply to routes:
   - Upload, approve, reject, publish: require 'admin' or 'social_manager' role.
   - Draft listing and published listing: require 'admin', 'social_manager', or 'viewer' role.
6. Export the middleware for use in route groups.

## Validation
Test with valid JWT and correct role → verify 200 response. Test with valid JWT but insufficient role → verify 403. Test with expired JWT → verify 401. Test with missing Authorization header → verify 401. Test with malformed token → verify 401.