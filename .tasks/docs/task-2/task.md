## Develop Equipment Catalog Service (Rex - Rust/Axum)

### Objective
Implement the high-performance Equipment Catalog Service, providing APIs for product inventory, availability, and image serving. This service is critical for Morgan's quoting capabilities and the website's product display.

### Ownership
- Agent: rex
- Stack: Rust/Axum
- Priority: high
- Status: pending
- Dependencies: 1

### Implementation Details
1. Initialize a new Rust project using `cargo new --bin equipment-catalog` targeting Rust 1.77.2.2. Set up Axum 0.7.5 with `tokio` runtime.3. Define `Product`, `Category`, and `Availability` data models using `sqlx` for PostgreSQL interaction. Implement database migrations for these schemas.4. Implement endpoints:    - `GET /api/v1/catalog/categories`    - `GET /api/v1/catalog/products` (filterable)    - `GET /api/v1/catalog/products/:id`    - `GET /api/v1/catalog/products/:id/availability?from=&to=`    - `POST /api/v1/catalog/products` (admin)    - `PATCH /api/v1/catalog/products/:id` (admin)    - `GET /api/v1/equipment-api/catalog` (machine-readable)    - `POST /api/v1/equipment-api/checkout` (programmatic booking)5. Integrate with PostgreSQL using `sqlx` and Redis for rate limiting and caching, referencing the `sigma1-infra-endpoints` ConfigMap.6. Implement S3/R2 integration for image serving, ensuring secure access and efficient retrieval.7. Add Prometheus metrics (`/metrics`), liveness (`/health/live`), and readiness (`/health/ready`) probes.8. Implement basic tenant-based rate limiting using Redis.

### Subtasks
- [ ] Initialize Rust project and configure Axum framework: Set up a new Rust project for the Equipment Catalog Service, configure Axum 0.7.5 with Tokio runtime, and establish a basic server structure.
- [ ] Define data models and implement database migrations: Define `Product`, `Category`, and `Availability` data models using `sqlx` and create initial database migration scripts for PostgreSQL.
- [ ] Integrate PostgreSQL and Redis for data and caching: Connect the service to PostgreSQL using `sqlx` and integrate Redis for caching and rate limiting, utilizing the `sigma1-infra-endpoints` ConfigMap.
- [ ] Implement core read-only catalog API endpoints: Develop the read-only API endpoints for categories, products, product details, and product availability, interacting with the PostgreSQL database.
- [ ] Implement admin and machine-readable catalog API endpoints: Develop API endpoints for administrative product management (create/update) and machine-readable catalog access, including programmatic booking.
- [ ] Implement S3/R2 image serving and service observability: Implement S3/R2 integration for secure and efficient image serving, and add Prometheus metrics, liveness, and readiness probes for service observability.