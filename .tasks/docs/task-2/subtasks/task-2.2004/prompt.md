Implement subtask 2004: Implement machine-readable Morgan API endpoints with rate limiting

## Objective
Implement the machine-readable API endpoints for the Morgan AI agent (/api/v1/equipment-api/catalog and /api/v1/equipment-api/checkout) and add Redis-based rate limiting middleware.

## Steps
1. Create a `routes/equipment_api.rs` module.
2. Implement `GET /api/v1/equipment-api/catalog`: return a simplified, structured JSON response optimized for LLM/agent consumption. Include fields: product_id, name, category, daily_rate, weekly_rate, monthly_rate, availability_summary (available_now: bool, next_available_date), image_url (primary). Support filtering by category and availability date range.
3. Implement `POST /api/v1/equipment-api/checkout`: accept a JSON body with { product_id, customer_name, customer_phone, rental_start, rental_end, notes }. Validate input, check availability for the date range, create a reservation record in rms.availability (mark dates as reserved), return confirmation with reservation_id. This is a simplified checkout for Morgan's use—no payment processing yet.
4. Create a `middleware/rate_limit.rs` module: implement a tower middleware or Axum layer that uses Redis INCR with TTL for sliding window rate limiting. Key by IP or API key. Default: 60 requests/minute for equipment-api endpoints.
5. Apply the rate limiting middleware to the /api/v1/equipment-api/* routes.
6. Wire routes into the main router.

## Validation
Call /equipment-api/catalog and verify simplified JSON structure suitable for agent consumption; POST to /equipment-api/checkout with valid data and verify reservation is created and availability is updated; POST with conflicting dates returns 409 Conflict; exceed 60 requests in 1 minute and verify 429 Too Many Requests response with Retry-After header.