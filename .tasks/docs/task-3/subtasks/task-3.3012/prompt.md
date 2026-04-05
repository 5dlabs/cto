Implement subtask 3012: Implement RBAC validation and JWT authentication middleware

## Objective
Create gRPC interceptors for JWT token validation from Authorization headers and RBAC role checking against the sigma1-rbac-roles ConfigMap.

## Steps
1. Create `internal/middleware/auth.go`:
   - Unary interceptor that extracts JWT from gRPC metadata 'authorization' key
   - Validate JWT signature (configurable: shared secret via env var or JWKS URL)
   - Extract claims: user_id, roles, exp
   - Reject expired tokens with Unauthenticated error
   - Inject validated claims into context for downstream handlers
2. Create `internal/middleware/rbac.go`:
   - Read `sigma1-rbac-roles` ConfigMap JSON on startup (path from env var, e.g., /etc/config/rbac-roles.json)
   - ConfigMap format: map of role names to arrays of allowed RPC method patterns (e.g., `{"admin": ["*"], "operator": ["/sigma1.rms.v1.OpportunityService/*", "/sigma1.rms.v1.ProjectService/*"]}`)
   - Unary interceptor that checks if any of the user's roles grant access to the current RPC method
   - Return PermissionDenied if no matching role
   - Watch file for changes and reload (fsnotify or periodic)
3. Create `internal/middleware/chain.go` to compose auth + RBAC interceptors in correct order (auth first, then RBAC).
4. Allow configurable bypass for health check RPCs.
5. Create stream interceptor variants for any streaming RPCs (future-proofing).

## Validation
Unit test: valid JWT passes auth interceptor and claims are in context. Expired JWT returns Unauthenticated. Missing token returns Unauthenticated. RBAC test: user with 'operator' role can access OpportunityService but not admin-only endpoints. User with 'admin' role can access everything. Health check RPCs bypass auth.