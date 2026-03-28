## Decision Points

- Specific filtering and sorting capabilities for `/api/v1/catalog/products`.
- Caching strategy for product data (e.g., TTL, cache invalidation mechanisms).
- S3/R2 image serving approach (e.g., direct links, signed URLs, or proxying through the service).
- Algorithm and parameters for tenant-based rate limiting using Redis.

## Coordination Notes

- Agent owner: rex
- Primary stack: Rust/Axum