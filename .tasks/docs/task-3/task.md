## Equipment Catalog Service - Availability & AI Agent API (Rex - Rust/Axum)

### Objective
Extend the Equipment Catalog service to include real-time availability checking, image serving from S3/R2, and machine-readable APIs for AI agents. This enables core functionality for quoting and programmatic booking.

### Ownership
- Agent: Rex
- Stack: Rust/Axum
- Priority: high
- Status: pending
- Dependencies: 1, 2

### Implementation Details
1. Extend the PostgreSQL schema to include the `Availability` data model (`product_id`, `date_from`, `date_to`, `quantity_available`, `reserved`, `booked`). 2. Implement the `GET /api/v1/catalog/products/:id/availability?from=&to=` endpoint to check product availability for a given date range. This should query the `Availability` table and consider `reserved` and `booked` quantities. 3. Implement the `GET /api/v1/equipment-api/catalog` endpoint, providing a machine-readable format of the entire catalog for AI agents. 4. Implement the `POST /api/v1/equipment-api/checkout` endpoint for programmatic booking, which should update `Availability` records. 5. Integrate S3/R2 for serving product images. Ensure image URLs stored in `Product` model are correctly resolved and served. 6. Integrate Redis for rate limiting on public endpoints and caching frequently accessed product data. Configure Redis connection using 'sigma1-infra-endpoints' ConfigMap. Use Rust 1.75+.

### Subtasks
- [ ] Implement Equipment Catalog Service - Availability & AI Agent API (Rex - Rust/Axum): Extend the Equipment Catalog service to include real-time availability checking, image serving from S3/R2, and machine-readable APIs for AI agents. This enables core functionality for quoting and programmatic booking.