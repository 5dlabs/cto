Implement subtask 2007: Implement rate limiting middleware using Redis

## Objective
Add a rate limiting middleware layer to the Axum router using Redis as the backing store, applying configurable limits per IP or API key.

## Steps
1. Create a middleware/rate_limit.rs module.
2. Implement an Axum middleware (tower Layer/Service) that:
   - Extracts the client identifier (IP address from X-Forwarded-For or direct connection, or API key from header).
   - Uses Redis INCR with TTL (sliding window or fixed window) to track request counts.
   - Returns 429 Too Many Requests with Retry-After header when limit exceeded.
   - Configurable limits: default 100 req/min for public endpoints, 1000 req/min for authenticated/agent endpoints.
3. Apply the middleware to the router (can be applied globally or per route group).
4. Add rate limit configuration to the AppState/config module.

## Validation
Send requests exceeding the configured rate limit from a single IP; verify 429 response is returned after the threshold. Verify Retry-After header is present. Verify requests under the limit succeed normally. Verify Redis keys are created with correct TTL.