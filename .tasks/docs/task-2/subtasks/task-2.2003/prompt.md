Implement subtask 2003: Implement JWT extraction and validation middleware for admin-protected write endpoints

## Objective
Write a reusable Axum middleware (or extractor) that extracts Bearer tokens from Authorization headers, validates them against JWT_SECRET, and enforces an 'admin' role claim for write endpoints.

## Steps
Create src/middleware/auth.rs. Use jsonwebtoken crate (add to Cargo.toml: jsonwebtoken = '9'). Define Claims struct: { sub: String, role: String, exp: usize }. Implement AuthExtractor as an axum::extract::FromRequestParts<Arc<AppState>> that: (1) reads Authorization header, (2) strips 'Bearer ' prefix, (3) decodes JWT with Validation::new(Algorithm::HS256) and JWT_SECRET from AppState, (4) returns 401 if missing or invalid, (5) returns extracted Claims. Implement AdminClaims newtype wrapping Claims that additionally checks role == 'admin', returning 403 if not. Add unit tests in src/middleware/auth.rs using #[cfg(test)] mod tests: create a valid token with jsonwebtoken::encode, pass it through AuthExtractor logic, verify Claims are returned. Test invalid token returns 401 error type.

## Validation
cargo test middleware::auth passes. Integration: POST /api/v1/catalog/products with no Authorization header returns HTTP 401. POST with valid non-admin token returns HTTP 403. POST with valid admin token proceeds to handler (returns 400 if body is invalid JSON, not 401/403).