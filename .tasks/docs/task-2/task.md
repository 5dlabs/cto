## Implement Equipment Catalog Service API (Rex - Rust/Axum)

### Objective
Develop the Equipment Catalog Service with endpoints for product/category listing, product details, availability checks, and machine-readable APIs for Morgan and other agents.

### Ownership
- Agent: Rex
- Stack: Rust 1.75+/Axum 0.7
- Priority: high
- Status: pending
- Dependencies: 1

### Implementation Details
{"steps": ["Initialize Rust project with Axum 0.7, connect to PostgreSQL and Redis using endpoints from ConfigMap.", "Define Product, Category, and Availability models as per PRD.", "Implement endpoints: /api/v1/catalog/categories, /api/v1/catalog/products, /api/v1/catalog/products/:id, /api/v1/catalog/products/:id/availability, /api/v1/equipment-api/catalog, /api/v1/equipment-api/checkout.", "Integrate S3/R2 for image URLs in product responses.", "Add rate limiting middleware using Redis.", "Implement Prometheus metrics and health endpoints.", "Write migrations for initial schema.", "Document OpenAPI spec for all endpoints."]}

### Subtasks
- [ ] Initialize Rust/Axum project with PostgreSQL and Redis connection pools: Scaffold the Rust project with Axum 0.7, configure connection pools for PostgreSQL (via sqlx) and Redis, reading endpoints from environment variables sourced from the sigma1-infra-endpoints ConfigMap.
- [ ] Define data models and create database migrations for catalog schema: Define Rust structs for Product, Category, and Availability domain models, and create sqlx migrations to set up the corresponding tables in the public (or rms) PostgreSQL schema.
- [ ] Implement category listing endpoint: Implement the GET /api/v1/catalog/categories endpoint that returns all equipment categories with optional tree structure support.
- [ ] Implement product listing and product detail endpoints: Implement GET /api/v1/catalog/products (with filtering and pagination) and GET /api/v1/catalog/products/:id for individual product details.
- [ ] Implement product availability endpoint: Implement GET /api/v1/catalog/products/:id/availability that returns availability data for a specific product within a date range.
- [ ] Implement machine-readable API endpoints for Morgan agent: Implement GET /api/v1/equipment-api/catalog and POST /api/v1/equipment-api/checkout endpoints designed for programmatic consumption by the Morgan AI agent.
- [ ] Implement rate limiting middleware using Redis: Add a rate limiting middleware layer to the Axum router using Redis as the backing store, applying configurable limits per IP or API key.
- [ ] Implement health check and Prometheus metrics endpoints: Add /healthz, /readyz, and /metrics endpoints for Kubernetes probes and Prometheus scraping.
- [ ] Generate OpenAPI specification and documentation: Generate a comprehensive OpenAPI 3.0 specification documenting all Equipment Catalog Service endpoints, request/response schemas, and error codes.
- [ ] Write integration tests for all catalog endpoints: Create comprehensive integration tests covering all Equipment Catalog Service endpoints with database seeding, happy paths, and error cases.