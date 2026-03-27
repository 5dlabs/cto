Implement subtask 2001: Implement Develop Equipment Catalog Service (Rex - Rust/Axum)

## Objective
Implement the high-performance Equipment Catalog Service, providing APIs for product inventory, availability, and image serving. This service is critical for Morgan's quoting capabilities and the website's product display.

## Steps
1. Initialize a new Rust project using `cargo new --bin equipment-catalog` targeting Rust 1.77.2.
2. Set up Axum 0.7.5 with `tokio` runtime.
3. Define `Product`, `Category`, and `Availability` data models using `sqlx` for PostgreSQL interaction. Implement database migrations for these schemas.
4. Implement endpoints:
    - `GET /api/v1/catalog/categories`
    - `GET /api/v1/catalog/products` (filterable)
    - `GET /api/v1/catalog/products/:id`
    - `GET /api/v1/catalog/products/:id/availability?from=&to=`
    - `POST /api/v1/catalog/products` (admin)
    - `PATCH /api/v1/catalog/products/:id` (admin)
    - `GET /api/v1/equipment-api/catalog` (machine-readable)
    - `POST /api/v1/equipment-api/checkout` (programmatic booking)
5. Integrate with PostgreSQL using `sqlx` and Redis for rate limiting and caching, referencing the `sigma1-infra-endpoints` ConfigMap.
6. Implement S3/R2 integration for image serving, ensuring secure access and efficient retrieval.
7. Add Prometheus metrics (`/metrics`), liveness (`/health/live`), and readiness (`/health/ready`) probes.
8. Implement basic tenant-based rate limiting using Redis.

## Validation
1. Deploy the service to Kubernetes and verify it starts successfully.
2. Use `curl` or Postman to verify all defined API endpoints are accessible and return expected data structures.
3. Test product creation, retrieval, and update operations.
4. Verify availability checks return correct quantities for specified date ranges.
5. Confirm rate limiting is active and blocks excessive requests.
6. Ensure product images are correctly uploaded to and retrieved from S3/R2.
7. Validate Prometheus metrics endpoint is exposed and returns data.