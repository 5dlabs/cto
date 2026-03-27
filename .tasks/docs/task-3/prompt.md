Implement task 3: Equipment Catalog Service - Availability & AI Agent API (Rex - Rust/Axum)

## Goal
Extend the Equipment Catalog service to include real-time availability checking, image serving from S3/R2, and machine-readable APIs for AI agents. This enables core functionality for quoting and programmatic booking.

## Task Context
- Agent owner: Rex
- Stack: Rust/Axum
- Priority: high
- Dependencies: 1, 2

## Implementation Plan
1. Extend the PostgreSQL schema to include the `Availability` data model (`product_id`, `date_from`, `date_to`, `quantity_available`, `reserved`, `booked`). 2. Implement the `GET /api/v1/catalog/products/:id/availability?from=&to=` endpoint to check product availability for a given date range. This should query the `Availability` table and consider `reserved` and `booked` quantities. 3. Implement the `GET /api/v1/equipment-api/catalog` endpoint, providing a machine-readable format of the entire catalog for AI agents. 4. Implement the `POST /api/v1/equipment-api/checkout` endpoint for programmatic booking, which should update `Availability` records. 5. Integrate S3/R2 for serving product images. Ensure image URLs stored in `Product` model are correctly resolved and served. 6. Integrate Redis for rate limiting on public endpoints and caching frequently accessed product data. Configure Redis connection using 'sigma1-infra-endpoints' ConfigMap. Use Rust 1.75+.

## Acceptance Criteria
1. Verify `GET /api/v1/catalog/products/:id/availability` returns correct availability data for various date ranges and product IDs, considering existing bookings. 2. Confirm `GET /api/v1/equipment-api/catalog` returns a comprehensive, machine-readable catalog. 3. Execute `POST /api/v1/equipment-api/checkout` with valid data and verify that product availability is correctly reduced in the database. 4. Upload a test image to S3/R2 and verify its URL is correctly served via the product details endpoint. 5. Test rate limiting by making multiple rapid requests to a public endpoint and observe expected 429 responses. 6. Monitor Redis cache hits for frequently accessed data. 7. Run `cargo audit` to check for known vulnerabilities in dependencies.

## Subtasks
- Implement Equipment Catalog Service - Availability & AI Agent API (Rex - Rust/Axum): Extend the Equipment Catalog service to include real-time availability checking, image serving from S3/R2, and machine-readable APIs for AI agents. This enables core functionality for quoting and programmatic booking.

## Deliverables
- Update the relevant code, configuration, and tests.
- Keep artifacts aligned with the acceptance criteria.
- Document blockers or assumptions in your final summary.