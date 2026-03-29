## Decision Points

- Image Storage Strategy: Decide on the specific S3/R2 bucket structure and access patterns for images.
- Rate Limiting Algorithm: Choose the specific algorithm and parameters for tenant-based rate limiting (e.g., token bucket, leaky bucket).
- Caching Strategy: Define what data to cache in Redis and for how long (e.g., product lists, availability).
- API Versioning Strategy: Confirm the API versioning strategy (`/api/v1/`) and future upgrade path.

## Coordination Notes

- Agent owner: rex
- Primary stack: Rust/Axum