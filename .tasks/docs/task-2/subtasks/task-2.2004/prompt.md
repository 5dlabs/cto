Implement subtask 2004: Implement shared-auth crate with JWT validation and RBAC middleware

## Objective
Build the shared-auth crate providing JWT token validation middleware and RBAC role-checking for Axum, reading role definitions from the sigma1-rbac-roles ConfigMap.

## Steps
1. Create `crates/shared-auth/Cargo.toml` depending on jsonwebtoken, axum (from workspace), axum-extra (for TypedHeader), serde, shared-error, tracing.
2. Define `AuthConfig` struct: jwt_secret (String), read from `JWT_SECRET` env var.
3. Define `Claims` struct matching JWT payload: sub (String), email (Option<String>), roles (Vec<String>), exp (usize), iat (usize).
4. Implement `pub async fn auth_middleware(request: Request, next: Next) -> Result<Response>` — extracts Bearer token from Authorization header, validates JWT signature and expiry, injects Claims into request extensions. Returns 401 on missing/invalid token.
5. Implement `pub fn require_role(role: &'static str) -> impl Fn(Request, Next) -> ... ` — an Axum middleware factory that checks if Claims in request extensions contains the specified role, returns 403 if not.
6. Load RBAC role mappings from `RBAC_ROLES_JSON` env var (mounted from ConfigMap). Parse into a HashMap<String, Vec<String>> mapping role names to permitted endpoints/actions. Use this in require_role checks.
7. Export extractors: `AuthUser` extractor that pulls Claims from request extensions for use in handlers.

## Validation
Unit tests: (1) Valid JWT decodes to correct Claims. (2) Expired JWT returns 401. (3) Missing Authorization header returns 401. (4) Claims with 'admin' role passes require_role('admin'). (5) Claims without 'admin' role returns 403 from require_role('admin'). (6) AuthUser extractor correctly pulls Claims from extensions.