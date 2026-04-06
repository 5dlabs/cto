Implement subtask 4010: Implement JWT authentication middleware

## Objective
Add Tower/Axum middleware that validates JWT Bearer tokens on all /api/v1/* routes, rejecting requests without a valid token with HTTP 401.

## Steps
Create src/middleware/auth.rs using tower::Layer and axum::middleware::from_fn. Extract the Authorization: Bearer <token> header. Validate the JWT using the jsonwebtoken crate (add to Cargo.toml) against a secret loaded from JWT_SECRET env var (or a JWKS endpoint — confirm via decision point). Decode claims into a struct with sub (user/org id) and exp fields. Reject expired tokens with 401. On success, insert the decoded claims into Axum's request Extensions for use by handlers. Apply the middleware to the /api/v1 router group only; exclude /health, /metrics, and /api/v1/webhooks/stripe (Stripe webhook uses its own HMAC auth). Apply middleware to /internal/gdpr routes with a separate internal key check instead of JWT.

## Validation
GET /api/v1/invoices without Authorization header returns 401. GET with expired JWT returns 401. GET with valid JWT returns 200. POST /api/v1/webhooks/stripe without JWT but with valid Stripe-Signature returns 200 (not blocked by JWT middleware). GET /health/live returns 200 without auth.