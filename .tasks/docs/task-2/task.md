## Implement Equipment Catalog Service API (Rex - Rust/Axum)

### Objective
Develop the Equipment Catalog service with all specified endpoints for product/category listing, availability, and machine-readable APIs. Enables Morgan and the website to access real-time inventory and quoting.

### Ownership
- Agent: rex
- Stack: Rust/Axum
- Priority: high
- Status: pending
- Dependencies: 1

### Implementation Details
{"steps": ["Initialize Rust 1.75+ project with Axum 0.7, sqlx for PostgreSQL, and redis-rs for Redis integration.", "Define data models for Product, Category, and Availability as per PRD.", "Implement endpoints: /api/v1/catalog/categories, /api/v1/catalog/products, /api/v1/catalog/products/:id, /api/v1/catalog/products/:id/availability, /api/v1/equipment-api/catalog, /api/v1/equipment-api/checkout, /metrics, /health/live, /health/ready.", "Integrate S3/R2 for image URLs and serve via CDN.", "Implement rate limiting per tenant using Redis.", "Add Prometheus metrics and health probes.", "Reference connection strings from 'sigma1-infra-endpoints' ConfigMap via envFrom.", "Write integration and unit tests for all endpoints."]}


### Subtasks
- [ ] Scaffold Rust project with Axum 0.7, sqlx, redis-rs, and configuration layer: Initialize the equipment-catalog Rust project with all required dependencies, project structure, configuration loading from environment variables (sigma1-infra-endpoints ConfigMap via envFrom), database connection pool, Redis connection, and application entrypoint.
- [ ] Define data models and create database migrations for Product, Category, and Availability: Create sqlx migrations for the rms schema defining categories, products, product_images, and availability tables, along with Rust struct definitions with serde serialization for API responses.
- [ ] Implement core catalog CRUD endpoints (categories, products, availability): Implement the public REST API endpoints for browsing the equipment catalog: list categories, list/filter products, get product by ID, and check product availability.
- [ ] Implement machine-readable equipment API endpoints with S3/R2 image URL integration: Implement the /equipment-api/catalog and /equipment-api/checkout endpoints designed for machine consumption by the Morgan agent, with full S3/R2 CDN image URL construction.
- [ ] Implement rate limiting middleware using Redis: Add per-tenant/per-IP rate limiting to all API endpoints using Redis as the backing store, with configurable limits for public vs. machine-readable endpoints.
- [ ] Add Prometheus metrics endpoint and Kubernetes health probes: Implement /metrics endpoint exposing Prometheus-format metrics, and /health/live and /health/ready endpoints for Kubernetes liveness and readiness probes.
- [ ] Write comprehensive integration and unit tests for all endpoints: Create a full test suite covering all catalog and equipment-api endpoints, error cases, pagination, rate limiting, and health probes to achieve at least 80% code coverage.