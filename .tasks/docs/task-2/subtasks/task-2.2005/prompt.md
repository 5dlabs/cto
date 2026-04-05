Implement subtask 2005: Implement shared API key validation middleware

## Objective
Build an API key authentication middleware in the shared crate that validates keys from the Authorization header against values in the sigma1-service-api-keys secret.

## Steps
1. In `shared/src/auth.rs`, implement API key extraction from `Authorization: Bearer <key>` header.
2. Define `ApiKeyStore` struct that loads keys from environment variables (injected from sigma1-service-api-keys secret). Keys should map to service identifiers (e.g., `CATALOG_ADMIN_KEY=xyz`, `FINANCE_SERVICE_KEY=abc`).
3. Implement Axum middleware `pub fn api_key_auth_layer(store: ApiKeyStore) -> impl Layer` that:
   - Extracts Bearer token from Authorization header
   - Looks up token in the ApiKeyStore
   - If valid, inserts the authenticated service identity into request extensions
   - If missing or invalid, returns 401 Unauthorized with standard error JSON
4. Provide a helper extractor `pub struct AuthenticatedService(pub String)` that can be used in handlers.
5. Support role-based checks: `pub fn require_role(role: &str) -> impl Layer` for admin-only routes.

## Validation
Unit test: valid API key returns 200 and injects AuthenticatedService. Missing header returns 401. Invalid key returns 401. Test require_role rejects keys without the required role. Test multiple keys are supported simultaneously.